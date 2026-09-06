//! matchbox `WebRtcSocket` behind the [`super::Transport`] API

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use futures::FutureExt;
use futures_timer::Delay;
use matchbox_socket::{WebRtcSocket, WebRtcSocketBuilder};
use tracing::info;

use super::{Packet, PeerId, PeerState};

pub struct WebRtcTransport {
    socket: WebRtcSocket,
    routine: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl WebRtcTransport {
    pub fn connect(room_url: &str, integrity_hash: Option<String>) -> Self {
        let mut builder = WebRtcSocketBuilder::new(room_url);
        if let Some(hash) = integrity_hash {
            builder = builder.integrity_hash(hash);
        }
        let (socket, loop_fut) = builder
            .signaling_keep_alive_interval(Some(Duration::from_secs(15)))
            .add_reliable_channel()
            .build();

        let handle = std::thread::Builder::new()
            .name("matchbox-loop-future_thread".to_string())
            .spawn(move || {
                crate::get_runtime().block_on(async {
                    let loop_fut = loop_fut.fuse();
                    futures::pin_mut!(loop_fut);

                    let timeout = Delay::new(Duration::from_millis(100));
                    futures::pin_mut!(timeout);

                    loop {
                        futures::select! {
                            _ = (&mut timeout).fuse() => {
                                timeout.reset(Duration::from_millis(100));
                            }

                            _ = &mut loop_fut => {
                                info!("[Network] WebRTC socket closed");
                                break;
                            }
                        }
                    }
                });
            })
            .expect("[Network] Failed to start WebRTC socket");

        Self {
            socket,
            routine: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn id(&mut self) -> Option<PeerId> {
        self.socket.id()
    }

    pub fn update_peers(&mut self) -> Vec<(PeerId, PeerState)> {
        let Ok(changes) = self.socket.try_update_peers() else {
            return Vec::new();
        };
        changes
            .into_iter()
            .map(|(peer, state)| {
                let state = match state {
                    matchbox_socket::PeerState::Connected => PeerState::Connected,
                    matchbox_socket::PeerState::Disconnected => PeerState::Disconnected,
                };
                (peer, state)
            })
            .collect()
    }

    pub fn connected_peers_count(&self) -> usize {
        self.socket.connected_peers().count()
    }

    pub fn send(&mut self, packet: Packet, peer: PeerId) {
        self.socket.channel_mut(0).send(packet, peer);
    }

    pub fn receive(&mut self) -> Vec<(PeerId, Packet)> {
        self.socket.channel_mut(0).receive()
    }

    pub fn is_closed(&self) -> bool {
        self.socket.channel(0).is_closed() || !self.is_routine_running()
    }

    pub fn close(&mut self) {
        self.socket.close();
    }

    pub fn is_routine_running(&self) -> bool {
        self.routine
            .lock()
            .map(|h| h.as_ref().map(|h| !h.is_finished()).unwrap_or(false))
            .unwrap_or(false)
    }
}
