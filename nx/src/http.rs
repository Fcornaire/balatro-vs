use std::io::{ErrorKind, Read, Write};
use std::time::{Duration, Instant};

use crate::tls::TlsStream;

const MAX_REDIRECTS: usize = 5;
/// Longest wait for more body bytes before giving up on the response
const READ_STALL: Duration = Duration::from_secs(30);

struct Url<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
}

fn parse_url(url: &str) -> Result<Url<'_>, String> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| format!("only https:// is supported: {url}"))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().map_err(|_| format!("bad port in {url}"))?),
        None => (authority, 443),
    };
    Ok(Url { host, port, path })
}

struct Response {
    status: u16,
    location: Option<String>,
    body: Vec<u8>,
}

fn read_to_end(stream: &mut TlsStream) -> Result<Vec<u8>, String> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 16 * 1024];
    let mut last_progress = Instant::now();
    loop {
        match stream.read(&mut buf) {
            Ok(0) => return Ok(raw),
            Ok(n) => {
                raw.extend_from_slice(&buf[..n]);
                last_progress = Instant::now();
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                if last_progress.elapsed() > READ_STALL {
                    return Err("response stalled".into());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) if !raw.is_empty() => return Ok(raw),
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn request(url: &Url) -> Result<Response, String> {
    let mut stream = TlsStream::connect(url.host, url.port)?;
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: balatro-vs\r\nAccept: */*\r\nConnection: close\r\n\r\n",
        url.path, url.host
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;
    let raw = read_to_end(&mut stream)?;

    let header_end = find(&raw, b"\r\n\r\n").ok_or("malformed HTTP response")?;
    let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
    let mut lines = head.split("\r\n");
    let status: u16 = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .ok_or("malformed status line")?;
    let mut location = None;
    let mut chunked = false;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            match k.trim().to_ascii_lowercase().as_str() {
                "location" => location = Some(v.trim().to_string()),
                "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                _ => {}
            }
        }
    }
    let body = raw[header_end + 4..].to_vec();
    let body = if chunked { dechunk(&body)? } else { body };

    Ok(Response {
        status,
        location,
        body,
    })
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn dechunk(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(data.len());
    let mut pos = 0;
    loop {
        let line_end = find(&data[pos..], b"\r\n").ok_or("malformed chunk header")? + pos;
        let size_str =
            std::str::from_utf8(&data[pos..line_end]).map_err(|_| "malformed chunk size")?;
        let size = usize::from_str_radix(size_str.split(';').next().unwrap_or("").trim(), 16)
            .map_err(|_| "malformed chunk size")?;
        pos = line_end + 2;
        if size == 0 {
            return Ok(out);
        }
        let end = pos + size;
        if end > data.len() {
            return Err("truncated chunk".into());
        }
        out.extend_from_slice(&data[pos..end]);
        pos = end + 2;
    }
}

pub fn get(url: &str) -> Result<Vec<u8>, String> {
    let mut current = url.to_string();
    for _ in 0..=MAX_REDIRECTS {
        let parsed = parse_url(&current)?;
        let res = request(&parsed)?;
        match res.status {
            200..=299 => return Ok(res.body),
            301 | 302 | 303 | 307 | 308 => {
                current = res
                    .location
                    .ok_or_else(|| format!("redirect without Location from {current}"))?;
            }
            s => {
                let snippet =
                    String::from_utf8_lossy(&res.body[..res.body.len().min(200)]).into_owned();
                return Err(format!("HTTP {s} for {current}: {snippet}"));
            }
        }
    }
    Err(format!("too many redirects from {url}"))
}
