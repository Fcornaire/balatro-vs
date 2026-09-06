//! How packets reach the opponent, `webrtc` or `relay`

#[cfg(feature = "relay")]
pub mod relay;
#[cfg(feature = "webrtc")]
pub mod webrtc;

pub use matchbox_protocol::PeerId;

/// The raw format of data being sent and received
pub type Packet = Box<[u8]>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerState {
    Connected,
    Disconnected,
}

/// WebRTC where matchbox is available, the relay elsewhere (Switch for example)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportKind {
    WebRtc,
    Relay,
}

impl TransportKind {
    pub fn for_target() -> Self {
        if cfg!(target_os = "switch") || !cfg!(feature = "webrtc") {
            TransportKind::Relay
        } else {
            TransportKind::WebRtc
        }
    }
}

pub enum Transport {
    #[cfg(feature = "webrtc")]
    WebRtc(webrtc::WebRtcTransport),
    #[cfg(feature = "relay")]
    Relay(relay::RelayClient),
}

pub fn connect(
    kind: TransportKind,
    room_url: &str,
    integrity_hash: Option<String>,
) -> Result<Transport, String> {
    match kind {
        #[cfg(feature = "webrtc")]
        TransportKind::WebRtc => Ok(Transport::WebRtc(webrtc::WebRtcTransport::connect(
            room_url,
            integrity_hash,
        ))),
        #[cfg(feature = "relay")]
        TransportKind::Relay => {
            relay::RelayClient::connect(room_url, integrity_hash).map(Transport::Relay)
        }
        #[allow(unreachable_patterns)]
        other => Err(format!("transport {other:?} is not compiled in")),
    }
}

macro_rules! dispatch {
    ($self:expr, $t:ident => $body:expr) => {
        match $self {
            #[cfg(feature = "webrtc")]
            Transport::WebRtc($t) => $body,
            #[cfg(feature = "relay")]
            Transport::Relay($t) => $body,
        }
    };
}

impl Transport {
    /// Our id on the signaling server
    pub fn id(&mut self) -> Option<PeerId> {
        dispatch!(self, t => t.id())
    }

    /// Peer state changes since the last call
    pub fn update_peers(&mut self) -> Vec<(PeerId, PeerState)> {
        dispatch!(self, t => t.update_peers())
    }

    pub fn connected_peers_count(&self) -> usize {
        dispatch!(self, t => t.connected_peers_count())
    }

    /// Send on the (only) reliable channel
    pub fn send(&mut self, packet: Packet, peer: PeerId) {
        dispatch!(self, t => t.send(packet, peer))
    }

    /// Packets received since the last call
    pub fn receive(&mut self) -> Vec<(PeerId, Packet)> {
        dispatch!(self, t => t.receive())
    }

    /// True once the connection to the signaling server / peers is gone
    pub fn is_closed(&self) -> bool {
        dispatch!(self, t => t.is_closed())
    }

    pub fn close(&mut self) {
        dispatch!(self, t => t.close())
    }

    /// True while the background routine (signaling / relay loop) is still running
    pub fn is_routine_running(&self) -> bool {
        dispatch!(self, t => t.is_routine_running())
    }
}
