//! Relay transport, a WebSocket to the matchbox server

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, OnceLock};

use matchbox_protocol::{JsonPeerRequest, PeerRequest};
use serde_json::json;
use tracing::warn;
use tungstenite::client::IntoClientRequest;

use super::{Packet, PeerId, PeerState};

mod worker;

static TLS_CONNECTOR: OnceLock<TlsConnector> = OnceLock::new();

/// Byte stream under the WebSocket: TCP for `ws://`, the platform's TLS for `wss://`.
pub trait Stream: Read + Write + Send {
    fn set_nonblocking(&mut self, nonblocking: bool) -> std::io::Result<()>;
}

impl Stream for TcpStream {
    fn set_nonblocking(&mut self, nonblocking: bool) -> std::io::Result<()> {
        TcpStream::set_nonblocking(self, nonblocking)
    }
}

pub type TlsConnector = fn(host: &str, port: u16) -> Result<Box<dyn Stream>, String>;

pub fn set_tls_connector(connector: TlsConnector) {
    let _ = TLS_CONNECTOR.set(connector);
}

enum Event {
    IdAssigned(PeerId),
    Peer(PeerId, PeerState),
    Packet(PeerId, Packet),
}

pub struct RelayClient {
    id: Option<PeerId>,
    events: Receiver<Event>,
    outgoing: Sender<JsonPeerRequest>,
    closed: Arc<AtomicBool>,
    connected: Vec<PeerId>,
    peer_changes: Vec<(PeerId, PeerState)>,
    inbox: Vec<(PeerId, Packet)>,
}

impl RelayClient {
    pub fn connect(room_url: &str, integrity_hash: Option<String>) -> Result<Self, String> {
        let url = if room_url.contains('?') {
            format!("{room_url}&relay=1")
        } else {
            format!("{room_url}?relay=1")
        };

        let mut request = url
            .clone()
            .into_client_request()
            .map_err(|e| format!("bad room url {url}: {e}"))?;

        if let Some(hash) = integrity_hash {
            request.headers_mut().insert(
                "x-Integrity-Hash",
                hash.parse()
                    .map_err(|e| format!("bad integrity hash: {e}"))?,
            );
        }

        let (events_tx, events_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel();
        let closed = Arc::new(AtomicBool::new(false));

        worker::submit(worker::Job {
            url,
            request,
            events: events_tx,
            outgoing: out_rx,
            closed: closed.clone(),
        })?;

        Ok(Self {
            id: None,
            events: events_rx,
            outgoing: out_tx,
            closed,
            connected: Vec::new(),
            peer_changes: Vec::new(),
            inbox: Vec::new(),
        })
    }

    fn receives(&mut self) {
        loop {
            match self.events.try_recv() {
                Ok(Event::IdAssigned(id)) => self.id = Some(id),
                Ok(Event::Peer(peer, state)) => {
                    match state {
                        PeerState::Connected => {
                            if !self.connected.contains(&peer) {
                                self.connected.push(peer);
                            }
                        }
                        PeerState::Disconnected => self.connected.retain(|p| *p != peer),
                    }
                    self.peer_changes.push((peer, state));
                }
                Ok(Event::Packet(peer, packet)) => self.inbox.push((peer, packet)),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.closed.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    pub fn id(&mut self) -> Option<PeerId> {
        self.receives();
        self.id
    }

    pub fn update_peers(&mut self) -> Vec<(PeerId, PeerState)> {
        self.receives();
        std::mem::take(&mut self.peer_changes)
    }

    pub fn connected_peers_count(&self) -> usize {
        self.connected.len()
    }

    pub fn send(&mut self, packet: Packet, peer: PeerId) {
        let request = PeerRequest::Signal {
            receiver: peer,
            data: json!({ "Relay": { "channel": 0, "data": packet.to_vec() } }),
        };
        if self.outgoing.send(request).is_err() {
            warn!("[Relay] send to {peer} dropped: connection is gone");
            self.closed.store(true, Ordering::Relaxed);
        }
    }

    pub fn receive(&mut self) -> Vec<(PeerId, Packet)> {
        self.receives();
        std::mem::take(&mut self.inbox)
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }

    pub fn is_routine_running(&self) -> bool {
        !self.closed.load(Ordering::Relaxed)
    }
}

impl Drop for RelayClient {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}
