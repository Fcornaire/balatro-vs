//! TLS stream over the console's `nn::ssl`

use std::io::{self, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Once;

use skyline::nn::socket::{self, SockAddrIn};

const AF_INET: i32 = 2;
const SOCK_STREAM: i32 = 1;

//From Ryujinx source
const RESULT_WOULD_BLOCK: u32 = (204 << 9) | 123;
const RESULT_TIMEOUT: u32 = (205 << 9) | 123;
const RESULT_CONNECTION_RESET: u32 = (209 << 9) | 123;
const RESULT_CONNECTION_ABORT: u32 = (210 << 9) | 123;

const IO_MODE_BLOCKING: u32 = 1;
const IO_MODE_NON_BLOCKING: u32 = 2;
const SSL_VERSION_AUTO: u32 = 1;

extern "C" {
    #[link_name = "\u{1}_ZN2nn6socket5CloseEi"]
    fn socket_close(fd: i32) -> i32;
}

mod ssl {
    use core::ffi::c_char;

    #[repr(C, align(8))]
    pub struct Context(pub [u8; 0x40]);
    #[repr(C, align(8))]
    pub struct Connection(pub [u8; 0x100]);

    extern "C" {
        #[link_name = "\u{1}_ZN2nn3ssl10InitializeEv"]
        pub fn Initialize() -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl7ContextC1Ev"]
        pub fn ContextNew(this: *mut Context);
        #[link_name = "\u{1}_ZN2nn3ssl7Context6CreateENS1_10SslVersionE"]
        pub fn ContextCreate(this: *mut Context, version: u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl7Context7DestroyEv"]
        pub fn ContextDestroy(this: *mut Context) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10ConnectionC1Ev"]
        pub fn ConnectionNew(this: *mut Connection);
        #[link_name = "\u{1}_ZN2nn3ssl10Connection6CreateEPNS0_7ContextE"]
        pub fn ConnectionCreate(this: *mut Connection, ctx: *mut Context) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection19SetSocketDescriptorEi"]
        pub fn SetSocketDescriptor(this: *mut Connection, fd: i32) -> i32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection11SetHostNameEPKcj"]
        pub fn SetHostName(this: *mut Connection, host: *const c_char, len: u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection9SetIoModeENS1_6IoModeE"]
        pub fn SetIoMode(this: *mut Connection, mode: u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection11DoHandshakeEv"]
        pub fn DoHandshake(this: *mut Connection) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection4ReadEPcPij"]
        pub fn Read(this: *mut Connection, buf: *mut u8, out_size: *mut i32, len: u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection5WriteEPKcPij"]
        pub fn Write(this: *mut Connection, buf: *const u8, out_size: *mut i32, len: u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection12GetLastErrorEPNS_6ResultE"]
        pub fn GetLastError(this: *mut Connection, out: *mut u32) -> u32;
        #[link_name = "\u{1}_ZN2nn3ssl10Connection7DestroyEv"]
        pub fn ConnectionDestroy(this: *mut Connection) -> u32;
    }
}

static SSL_INIT: Once = Once::new();

pub struct TlsStream {
    conn: Box<ssl::Connection>,
    ctx: Box<ssl::Context>,
}

fn tcp_connect(host: &str, port: u16) -> Result<i32, String> {
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("resolve {host}: {e}"))?
        .find_map(|a| match a {
            SocketAddr::V4(v4) => Some(v4),
            _ => None,
        })
        .ok_or_else(|| format!("no IPv4 address for {host}"))?;

    unsafe {
        let fd = socket::Socket(AF_INET, SOCK_STREAM, 0);
        if fd < 0 {
            return Err(format!("socket: errno {}", socket::GetLastError()));
        }

        let sa = SockAddrIn {
            sin_len: core::mem::size_of::<SockAddrIn>() as u8,
            sin_family: AF_INET as u8,
            sin_port: socket::InetHtons(port),
            sin_addr: addr.ip().octets(),
            padding: 0,
        };

        if socket::Connect(fd, &sa, core::mem::size_of::<SockAddrIn>() as u32) != 0 {
            let err = socket::GetLastError();
            socket_close(fd);
            return Err(format!(
                "connect {host}:{port} ({}): errno {err}",
                addr.ip()
            ));
        }

        Ok(fd)
    }
}

fn io_error(op: &str, rc: u32) -> io::Error {
    let kind = match rc {
        RESULT_WOULD_BLOCK => io::ErrorKind::WouldBlock,
        RESULT_TIMEOUT => io::ErrorKind::TimedOut,
        RESULT_CONNECTION_RESET => io::ErrorKind::ConnectionReset,
        RESULT_CONNECTION_ABORT => io::ErrorKind::ConnectionAborted,
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, format!("ssl {op}: {rc:#x}"))
}

impl TlsStream {
    pub fn connect(host: &str, port: u16) -> Result<Self, String> {
        SSL_INIT.call_once(|| unsafe {
            let rc = ssl::Initialize();
            if rc != 0 {
                println!("[bvs-nx] nn::ssl::Initialize -> {rc:#x}");
            }
        });

        let fd = tcp_connect(host, port)?;

        unsafe {
            let mut ctx = Box::new(ssl::Context([0; 0x40]));
            ssl::ContextNew(&mut *ctx);

            let rc = ssl::ContextCreate(&mut *ctx, SSL_VERSION_AUTO);
            if rc != 0 {
                socket_close(fd);
                return Err(format!("ssl Context::Create: {rc:#x}"));
            }

            let mut conn = Box::new(ssl::Connection([0; 0x100]));
            ssl::ConnectionNew(&mut *conn);
            let rc = ssl::ConnectionCreate(&mut *conn, &mut *ctx);
            if rc != 0 {
                ssl::ContextDestroy(&mut *ctx);
                socket_close(fd);
                return Err(format!("ssl Connection::Create: {rc:#x}"));
            }

            let mut stream = TlsStream { conn, ctx };
            ssl::SetSocketDescriptor(&mut *stream.conn, fd);
            let rc = ssl::SetHostName(
                &mut *stream.conn,
                host.as_ptr() as *const _,
                host.len() as u32,
            );

            if rc != 0 {
                return Err(format!("ssl SetHostName: {rc:#x}"));
            }

            let rc = ssl::DoHandshake(&mut *stream.conn);
            if rc != 0 {
                let mut last = 0u32;
                ssl::GetLastError(&mut *stream.conn, &mut last);
                return Err(format!(
                    "ssl handshake with {host}: {rc:#x} (last error {last:#x})"
                ));
            }
            Ok(stream)
        }
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        let mode = if nonblocking {
            IO_MODE_NON_BLOCKING
        } else {
            IO_MODE_BLOCKING
        };
        match unsafe { ssl::SetIoMode(&mut *self.conn, mode) } {
            0 => Ok(()),
            rc => Err(io_error("SetIoMode", rc)),
        }
    }
}

impl Drop for TlsStream {
    fn drop(&mut self) {
        unsafe {
            ssl::ConnectionDestroy(&mut *self.conn);
            ssl::ContextDestroy(&mut *self.ctx);
        }
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n: i32 = 0;
        match unsafe { ssl::Read(&mut *self.conn, buf.as_mut_ptr(), &mut n, buf.len() as u32) } {
            0 => Ok(n.max(0) as usize),
            rc => Err(io_error("read", rc)),
        }
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut n: i32 = 0;
        match unsafe { ssl::Write(&mut *self.conn, buf.as_ptr(), &mut n, buf.len() as u32) } {
            0 => Ok(n.max(0) as usize),
            rc => Err(io_error("write", rc)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl bvs_core::transport::relay::Stream for TlsStream {
    fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        TlsStream::set_nonblocking(self, nonblocking)
    }
}

pub fn relay_connect(
    host: &str,
    port: u16,
) -> Result<Box<dyn bvs_core::transport::relay::Stream>, String> {
    TlsStream::connect(host, port)
        .map(|s| Box::new(s) as Box<dyn bvs_core::transport::relay::Stream>)
}
