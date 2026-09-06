//! The relay worker, one long lived thread with non-blocking sockets serves every connection

use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use matchbox_protocol::{JsonPeerEvent, JsonPeerRequest};
use serde_json::Value;
use tracing::{debug, error, info, warn};
use tungstenite::{Message, WebSocket};

use super::{Event, Stream, TLS_CONNECTOR};
use crate::transport::{Packet, PeerState};

const KEEP_ALIVE: Duration = Duration::from_secs(15);

const IDLE_SLEEP_MIN: Duration = Duration::from_millis(2);
const IDLE_SLEEP_MAX: Duration = Duration::from_millis(40);

/// One connection to open, handed to the worker thread
pub(super) struct Job {
    pub(super) url: String,
    pub(super) request: tungstenite::handshake::client::Request,
    pub(super) events: Sender<Event>,
    pub(super) outgoing: Receiver<JsonPeerRequest>,
    pub(super) closed: Arc<AtomicBool>,
}

/// A live connection served by the worker
struct Conn {
    url: String,
    ws: WebSocket<Box<dyn Stream>>,
    events: Sender<Event>,
    outgoing: Receiver<JsonPeerRequest>,
    closed: Arc<AtomicBool>,
    last_keep_alive: Instant,
    pending_flush: bool,
}

/// Hands a connection to the worker thread
pub(super) fn submit(job: Job) -> Result<(), String> {
    static WORKER: OnceLock<Sender<Job>> = OnceLock::new();
    let jobs = WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("bvs-relay".to_string())
            .spawn(move || worker_loop(rx))
            .expect("cannot spawn the relay thread");
        tx
    });
    jobs.send(job)
        .map_err(|_| "relay worker thread is gone".to_string())
}

fn open_stream(
    request: &tungstenite::handshake::client::Request,
) -> Result<Box<dyn Stream>, String> {
    let uri = request.uri();
    let host = uri.host().ok_or("room url has no host")?;
    match uri.scheme_str() {
        Some("ws") => {
            let port = uri.port_u16().unwrap_or(80);
            TcpStream::connect((host, port))
                .map(|s| Box::new(s) as Box<dyn Stream>)
                .map_err(|e| format!("connect {host}:{port}: {e}"))
        }
        Some("wss") => {
            let port = uri.port_u16().unwrap_or(443);
            let connect = TLS_CONNECTOR
                .get()
                .ok_or("no TLS connector registered for wss://")?;
            connect(host, port)
        }
        other => Err(format!("unsupported scheme {other:?}")),
    }
}

fn open(job: Job) -> Option<Conn> {
    info!("[Relay] connecting to {}", job.url);
    let connected = open_stream(&job.request).and_then(|stream| {
        tungstenite::client(job.request, stream)
            .map(|(ws, _)| ws)
            .map_err(|e| format!("websocket handshake: {e}"))
    });
    match connected {
        Ok(mut ws) => {
            if let Err(e) = ws.get_mut().set_nonblocking(true) {
                warn!("[Relay] cannot switch the stream to non-blocking: {e}");
            }
            Some(Conn {
                url: job.url,
                ws,
                events: job.events,
                outgoing: job.outgoing,
                closed: job.closed,
                last_keep_alive: Instant::now(),
                pending_flush: false,
            })
        }
        Err(e) => {
            error!("[Relay] connection to {} failed: {e}", job.url);
            job.closed.store(true, Ordering::Relaxed);
            None
        }
    }
}

fn worker_loop(jobs: Receiver<Job>) {
    crate::run_thread_start_hook();
    let mut conns: Vec<Conn> = Vec::new();
    let mut idle = IDLE_SLEEP_MIN;
    loop {
        if conns.is_empty() {
            match jobs.recv() {
                Ok(job) => conns.extend(open(job)),
                Err(_) => loop {
                    std::thread::sleep(Duration::from_secs(60));
                },
            }
        }

        while let Ok(job) = jobs.try_recv() {
            conns.extend(open(job));
        }

        let mut busy = false;
        conns.retain_mut(|conn| {
            let (alive, progressed) = step(conn);
            busy |= progressed;
            if !alive {
                conn.closed.store(true, Ordering::Relaxed);
                info!("[Relay] connection to {} finished", conn.url);
            }
            alive
        });

        if busy {
            idle = IDLE_SLEEP_MIN;
        } else {
            std::thread::sleep(idle);
            idle = (idle * 2).min(IDLE_SLEEP_MAX);
        }
    }
}

fn would_block(e: &tungstenite::Error) -> bool {
    let tungstenite::Error::Io(io) = e else {
        return false;
    };

    if matches!(
        io.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ) {
        return true;
    }
    // Switch nn::socket reports EAGAIN as errno 1
    cfg!(target_os = "switch") && io.raw_os_error() == Some(1)
}

fn send_text(conn: &mut Conn, text: String) -> Result<(), tungstenite::Error> {
    match conn.ws.send(Message::Text(text)) {
        Ok(()) => Ok(()),
        Err(e) if would_block(&e) => {
            conn.pending_flush = true;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// One iteration for a connection, outgoing requests, keep-alive, ...
fn step(conn: &mut Conn) -> (bool, bool) {
    if conn.closed.load(Ordering::Relaxed) {
        let _ = conn.ws.close(None);
        let _ = conn.ws.flush();
        return (false, true);
    }
    let mut progressed = false;

    loop {
        match conn.outgoing.try_recv() {
            Ok(request) => {
                progressed = true;
                if let Err(e) = send_text(conn, request.to_string()) {
                    error!("[Relay] send failed: {e}");
                    return (false, true);
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => return (false, true),
        }
    }

    if conn.last_keep_alive.elapsed() >= KEEP_ALIVE {
        conn.last_keep_alive = Instant::now();
        if let Err(e) = send_text(conn, JsonPeerRequest::KeepAlive.to_string()) {
            error!("[Relay] keep-alive failed: {e}");
            return (false, true);
        }
    }

    if conn.pending_flush {
        match conn.ws.flush() {
            Ok(()) => conn.pending_flush = false,
            Err(e) if would_block(&e) => {}
            Err(e) => {
                error!("[Relay] flush failed: {e}");
                return (false, true);
            }
        }
        progressed = true;
    }

    match conn.ws.read() {
        Ok(Message::Text(text)) => match text.parse::<JsonPeerEvent>() {
            Ok(event) => (handle_event(event, &conn.events).is_ok(), true),
            Err(e) => {
                warn!("[Relay] unreadable event {text}: {e}");
                (true, true)
            }
        },
        Ok(Message::Close(_)) => {
            info!("[Relay] server closed the connection");
            (false, true)
        }
        Ok(Message::Ping(_)) | Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => (true, true),
        Ok(Message::Binary(_)) => {
            warn!("[Relay] ignoring binary message");
            (true, true)
        }
        Err(e) if would_block(&e) => (true, progressed),
        Err(tungstenite::Error::ConnectionClosed) | Err(tungstenite::Error::AlreadyClosed) => {
            info!("[Relay] connection closed");
            (false, true)
        }
        Err(e) => {
            error!("[Relay] read failed: {e}");
            (false, true)
        }
    }
}

fn handle_event(event: JsonPeerEvent, events: &Sender<Event>) -> Result<(), ()> {
    let send = |e: Event| events.send(e).map_err(|_| ());
    match event {
        JsonPeerEvent::IdAssigned(id) => {
            debug!("[Relay] id assigned: {id}");
            send(Event::IdAssigned(id))
        }
        JsonPeerEvent::NewRelayPeer(peer) => {
            info!("[Relay] paired with {peer}");
            send(Event::Peer(peer, PeerState::Connected))
        }
        JsonPeerEvent::NewPeer(peer) => {
            warn!("[Relay] server announced {peer} as a WebRTC peer, treating it as relay peer");
            send(Event::Peer(peer, PeerState::Connected))
        }
        JsonPeerEvent::PeerLeft(peer) => {
            info!("[Relay] {peer} left");
            send(Event::Peer(peer, PeerState::Disconnected))
        }
        JsonPeerEvent::Signal { sender, data } => match relay_payload(&data) {
            Some(packet) => send(Event::Packet(sender, packet)),
            None => {
                debug!("[Relay] ignoring non relay signal from {sender}");
                Ok(())
            }
        },
    }
}

fn relay_payload(data: &Value) -> Option<Packet> {
    let bytes = data.get("Relay")?.get("data")?.as_array()?;
    let packet: Option<Vec<u8>> = bytes.iter().map(|b| b.as_u64().map(|b| b as u8)).collect();
    packet.map(Vec::into_boxed_slice)
}
