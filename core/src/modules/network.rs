use bincode::{deserialize, serialize};
use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::{PeerId, PeerState, WebRtcSocketBuilder};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use crate::{get_bvs_config, init_or_reset_ws_routine_handle};
use crate::{
    get_runtime, get_ws, modules::game_manipulation::GameManipulationEvent, reset_ws, set_ws,
};

use super::integrity::get_integrity_hash;
use super::{card_conf::AreaType, game_manipulation::GameManipulation, CardConf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkEvent {
    NewPeer(String, String, String),
    ConfirmMatch,
    HighlightedCard(Vec<usize>),
    NewCardAlignement(Vec<usize>, String),
    SortedHandSuit,
    SortedHandValue,
    DiscardedCards(Vec<usize>),
    NewCardToEmplace(CardConf),
    WaitForYourAction,
    UsedConsumeableCard(usize, AreaType, Vec<usize>, Vec<CardConf>),
    VoucherUsed(CardConf),
    OpenBooster(CardConf, Vec<CardConf>),
    HighlightedBoosterCard(Vec<usize>),
    RerollShop,
    BoughtCard(CardConf, String),
    SellCard(usize, bool),
    Cashout(i32),
    Ping(),
    Pong(),
    Rematch(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkState {
    Idle,
    Search,
    Found,
    SelfConfirm,
    OpponentConfirm,
    Start,
    OpponentHighlighted(Vec<usize>),
    SentHighlighted(Vec<usize>),
    PlayTurn(Vec<usize>),
    WaitForUserAction,
    WaitForOpponent(GameManipulationEvent),
    OpponentWaitingForYou,
    WaitingForRematchResponse,
    OpponentRematched,
}

pub struct Network {
    state: NetworkState,
    connected_date: Option<OffsetDateTime>,
    opponent: Option<PeerId>,
    rtt: usize,
    friendly_room_code: String,
    ping_measurement: Instant,
    is_waiting_for_pong: bool,
}

impl Network {
    pub fn new() -> Self {
        Self {
            state: NetworkState::Idle,
            connected_date: None,
            opponent: None,
            rtt: 0,
            friendly_room_code: String::new(),
            ping_measurement: Instant::now(),
            is_waiting_for_pong: false,
        }
    }

    pub fn send_ping(&mut self) {
        self.ping_measurement = Instant::now();
        self.is_waiting_for_pong = true;

        if let Err(e) = self.send_event_to_opponent(NetworkEvent::Ping()) {
            error!("[Network] Send_ping : {:?}", e);
            return;
        }
    }

    fn send_event_to_opponent(&self, event: NetworkEvent) -> Result<(), String> {
        if let Some(peer) = &self.opponent {
            let packet = serialize(&event).unwrap().into_boxed_slice();
            let mut socket = get_ws().unwrap().lock().unwrap();
            let socket = socket.as_mut().unwrap();
            socket.channel_mut(0).send(packet, *peer);
            return Ok(());
        }

        Err("No opponent found".to_string())
    }

    fn connect(&self, room_code: String) {
        let server = get_bvs_config().get_server();
        let protocol = server.get_protocol();
        let port = if server.get_port() == 0 {
            "".to_string()
        } else {
            format!(":{}", server.get_port())
        };

        let mut builder = WebRtcSocketBuilder::new(&format!(
            "{}://{}{}{}{room_code}?next=2",
            protocol,
            server.get_host(),
            port,
            server.get_path()
        ));

        if cfg!(feature = "with_integrity") {
            builder = builder.integrity_hash(get_integrity_hash());
        }

        let (socket, loop_fut) = builder
            .signaling_keep_alive_interval(Some(Duration::from_secs(15)))
            .add_reliable_channel()
            .build();

        let handle = std::thread::Builder::new()
            .name("matchbox-loop-future_thread".to_string())
            .spawn(move || {
                get_runtime().block_on(async {
                    let loop_fut = loop_fut.fuse();
                    futures::pin_mut!(loop_fut);

                    let timeout = Delay::new(Duration::from_millis(100));
                    futures::pin_mut!(timeout);

                    loop {
                        select! {
                            _ = (&mut timeout).fuse() => {
                                timeout.reset(Duration::from_millis(100));
                            }

                            _ = &mut loop_fut => {
                                info!("[Network] WebRTC socket closed");
                            }
                        }
                    }
                });
            });

        if let Ok(handle) = handle {
            init_or_reset_ws_routine_handle(handle);
        } else {
            panic!("[Network] Failed to start WebRTC socket");
        }

        set_ws(socket);
    }

    pub fn start_matchmaking(&mut self, game_manipulation: &mut GameManipulation) -> bool {
        let socket = get_ws();
        if socket.is_none() || socket.as_ref().unwrap().lock().unwrap().is_none() {
            let room_code = "random".to_string();
            self.connect(room_code);
        }

        self.connected_date = Some(OffsetDateTime::now_utc());

        match self.state {
            NetworkState::Idle => {
                self.state = NetworkState::Search;
                game_manipulation
                    .register_event(GameManipulationEvent::OnRandomMatchmakingSelected);

                true
            }
            _ => false,
        }
    }

    pub fn start_versus_friendlies(&mut self) -> String {
        let socket = get_ws();
        if socket.is_none() || socket.as_ref().unwrap().lock().unwrap().is_none() {
            let room_code = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
            self.connect(room_code.clone());
            self.friendly_room_code = room_code.clone();
        }

        self.connected_date = Some(OffsetDateTime::now_utc());

        match self.state {
            NetworkState::Idle => {
                self.state = NetworkState::Search;
            }
            _ => {}
        }

        self.friendly_room_code.clone()
    }

    pub fn start_versus_friendlies_pairing(&mut self, room_code: String) -> bool {
        let socket = get_ws();
        if socket.is_none() || socket.as_ref().unwrap().lock().unwrap().is_none() {
            self.state = NetworkState::Search;

            info!("Starting friendly match with room code: {}", room_code);
            self.connect(room_code.clone());
            self.friendly_room_code = room_code.clone();
        }

        self.connected_date = Some(OffsetDateTime::now_utc());

        true
    }

    pub fn quit_server(&mut self) -> bool {
        debug!("[Network] Quit_matchmaking");

        let socket = get_ws();

        if socket.is_none() || socket.as_ref().unwrap().lock().unwrap().is_none() {
            warn!("[Network] No socket to close");
            return false;
        }

        let mut socket_guard = socket.unwrap().lock().unwrap();
        let socket = socket_guard.as_mut().unwrap();

        self.opponent = None;
        self.friendly_room_code.clear();
        socket.close();
        drop(socket_guard);
        reset_ws();

        self.state = NetworkState::Idle;

        true
    }

    pub fn confirm_versus_matchmaking(&mut self, game_manipulation: &mut GameManipulation) -> bool {
        debug!("[Network] Confirm_versus_matchmaking");
        if let Some(peer) = &self.opponent {
            let event = NetworkEvent::ConfirmMatch;
            let packet = serialize(&event).unwrap().into_boxed_slice();

            let mut socket = get_ws().unwrap().lock().unwrap();
            let socket = socket.as_mut().unwrap();
            socket.channel_mut(0).send(packet, *peer);

            match self.state {
                NetworkState::Found => {
                    self.state = NetworkState::SelfConfirm;
                }
                NetworkState::OpponentConfirm => {
                    self.state = NetworkState::Start;

                    game_manipulation.register_event(GameManipulationEvent::OnAcceptedRandomMath);
                }
                _ => {}
            }

            return true;
        }

        warn!("[Network] Confirm_versus_matchmaking : No opponent found");

        false
    }

    pub fn poll_and_update(&mut self, game_manipulation: &mut GameManipulation) {
        let socket = get_ws();
        if socket.is_none() || socket.as_ref().unwrap().lock().unwrap().is_none() {
            // No socket yet
            return;
        }

        let mut socket_guard = socket.unwrap().lock().unwrap();
        let socket = socket_guard.as_mut().unwrap();

        if socket.id().is_some() {
            // having an id means the connection is established to the server
            for (peer, state) in socket.update_peers() {
                match state {
                    PeerState::Connected => {
                        info!("Peer joined: {peer}");

                        let seed = game_manipulation.get_seed();

                        let event = NetworkEvent::NewPeer(
                            peer.to_string().clone(),
                            self.connected_date
                                .unwrap()
                                .format(&time::format_description::well_known::Rfc3339)
                                .unwrap(),
                            seed,
                        );
                        let packet = serialize(&event).unwrap().into_boxed_slice();

                        socket.channel_mut(0).send(packet, peer);
                    }
                    PeerState::Disconnected => {
                        info!("Peer left: {peer}");

                        if self.opponent.is_none() {
                            warn!(
                                "but no opponent found ? Ignoring disconnection (server fix needed probably)"
                            );
                            continue;
                        }

                        self.opponent = None;
                        socket.close();

                        game_manipulation.regenerate_seed();

                        match self.state {
                            NetworkState::Found
                            | NetworkState::OpponentConfirm
                            | NetworkState::SelfConfirm
                            | NetworkState::Idle => {
                                game_manipulation.register_event(
                                    GameManipulationEvent::OpponentDisconnectedBeforeConfirm,
                                );
                            }
                            _ => {
                                game_manipulation.register_event(
                                    GameManipulationEvent::OpponentDisconnectedInGame,
                                );
                            }
                        }

                        self.state = NetworkState::Idle;
                    }
                }
            }

            if !socket.channel_mut(0).is_closed() {
                if socket.connected_peers().count() > 0 {
                    // Accept any messages incoming
                    let received = socket.channel_mut(0).receive(); // Updated to use channel_mut(0)

                    drop(socket_guard);

                    received.iter().for_each(|(peer, packet)| {
                        self.on_messages_received(game_manipulation, &peer, &packet);
                    });

                    //Send ping every 3 seconds
                    if self.opponent.is_some() {
                        if self.ping_measurement.elapsed().as_millis() >= 3000
                            && !self.is_waiting_for_pong
                        {
                            self.send_ping();
                        }
                    }
                }
            } else {
                drop(socket_guard);
                reset_ws();
            }
        }

        //Process any game events
        game_manipulation.process_events();

        //Handle timer update
        game_manipulation.handle_timer_with_state_update(self.state.clone());
    }

    pub fn wait_for_next_action(&mut self) {
        debug!("[Network] Wait_for_next_action");
        self.state = NetworkState::WaitForUserAction;
    }

    pub fn send_to_opponent_new_cards_alignement(
        &mut self,
        alignement: Vec<usize>,
        _type: String,
    ) -> bool {
        debug!(
            "[Network] Send_to_opponent_new_cards_alignement : {:?}",
            alignement
        );
        let event = NetworkEvent::NewCardAlignement(alignement.clone(), _type);

        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Send_to_opponent_new_cards_alignement : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_highlighted_card(
        &mut self,
        game_manipulation: &mut GameManipulation,
        highlighted_cards: Vec<usize>,
    ) -> bool {
        debug!("[Network] Send_highlighted_card : {:?}", highlighted_cards);
        let event = NetworkEvent::HighlightedCard(highlighted_cards.clone());

        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Send_highlighted_card : {:?}", e);
            return false;
        }

        match &self.state {
            NetworkState::WaitForUserAction => {
                self.state = NetworkState::SentHighlighted(highlighted_cards);
                game_manipulation.register_event(GameManipulationEvent::UpdateMessage(
                    "Waiting for opponent".to_string(),
                ))
            }
            NetworkState::OpponentHighlighted(cards) => {
                game_manipulation
                    .register_event(GameManipulationEvent::UpdateMessage("".to_string()));
                game_manipulation.register_event(GameManipulationEvent::PlayTurn(cards.clone()));
                self.state = NetworkState::PlayTurn(cards.clone());
            }
            _ => {}
        }
        true
    }

    pub fn player_sort_hand_suit(&mut self) {
        debug!("[Network] Player_sort_hand_suit");

        let event = NetworkEvent::SortedHandSuit;
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_sort_hand_suit : {:?}", e);
            return;
        }
    }

    pub fn player_sort_hand_value(&mut self) {
        debug!("[Network] Player_sort_hand_value");

        let event = NetworkEvent::SortedHandValue;
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_sort_hand_value : {:?}", e);
            return;
        }
    }

    pub fn player_discarded_cards(
        &mut self,
        discarded_cards: Vec<usize>,
        game_manipulation: &mut GameManipulation,
    ) {
        debug!("[Network] Player_discarded_cards");

        let event = NetworkEvent::DiscardedCards(discarded_cards);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_discarded_cards : {:?}", e);
            return;
        }

        game_manipulation.register_event(GameManipulationEvent::DiscardedHandCards);
    }

    pub fn has_opponent_highlithed_cards(&self) -> bool {
        matches!(self.state, NetworkState::OpponentHighlighted(_))
    }

    pub fn send_new_card(&mut self, card_conf: CardConf) {
        debug!("[Network] send_new_card");

        let event = NetworkEvent::NewCardToEmplace(card_conf);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] send_new_card : {:?}", e);
            return;
        }
    }

    pub fn wait_for_opponent_action_before(
        &mut self,
        game_manipulation: &mut GameManipulation,
        game_event: GameManipulationEvent,
    ) {
        debug!("[Network] Wait_for_opponent_before_next_action");

        let event = NetworkEvent::WaitForYourAction;
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Wait_for_opponent_before_next_action : {:?}", e);
            return;
        }

        match &self.state {
            NetworkState::WaitForUserAction => {
                self.state = NetworkState::WaitForOpponent(game_event.clone());
                game_manipulation.register_event(GameManipulationEvent::UpdateMessage(
                    "Waiting for opponent".to_string(),
                ));
            }
            NetworkState::OpponentWaitingForYou => {
                game_manipulation.acknowledge_event();

                if let GameManipulationEvent::ProcessRemainingEvents = &(game_event.clone()) {
                    game_manipulation.register_event(GameManipulationEvent::UpdateMessage(
                        "Playing opponent shop choices".to_string(),
                    ));
                } else {
                    game_manipulation
                        .register_event(GameManipulationEvent::UpdateMessage("".to_string()));
                }

                game_manipulation.register_event(game_event.clone());
                self.state = NetworkState::WaitForUserAction;
            }
            _ => {
                warn!(
                    "[Network] Wrong state {:?} for Wait_for_opponent_before_next_action",
                    self.state
                );
            }
        }
    }

    pub fn player_use_consumeable_card(
        &mut self,
        index: usize,
        area_type: AreaType,
        cards_index: Vec<usize>,
        targets: Vec<CardConf>,
    ) -> bool {
        debug!("[Network] Player_use_consumeable_card");

        let event = NetworkEvent::UsedConsumeableCard(index, area_type, cards_index, targets);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_use_consumeable_card : {:?}", e);
            return false;
        }

        true
    }

    pub fn player_use_voucher_card(&mut self, card: CardConf) -> bool {
        debug!("[Network] Player_use_voucher_card");

        let event = NetworkEvent::VoucherUsed(card);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_use_voucher_card : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_open_booster(
        &mut self,
        card: CardConf,
        shop_jokers_cards_conf: Vec<CardConf>,
    ) -> bool {
        debug!("[Network] Send_open_booster");

        let event = NetworkEvent::OpenBooster(card, shop_jokers_cards_conf);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Send_open_booster : {:?}", e);
            return false;
        }

        true
    }

    pub fn player_skip_booster(&mut self) -> bool {
        debug!("[Network] Player_skip_booster");

        let event = NetworkEvent::HighlightedBoosterCard(vec![]);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_skip_booster : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_new_card_from_booster(&mut self, card_index: usize) -> bool {
        debug!("[Network] Send_new_card_from_booster");

        let event = NetworkEvent::HighlightedBoosterCard(vec![card_index]);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Send_new_card_from_booster : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_reroll_shop(&mut self) -> bool {
        debug!("[Network] Send_reroll_shop");

        let event = NetworkEvent::RerollShop;
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Send_reroll_shop : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_bought_card(&mut self, card: CardConf, id: String) -> bool {
        debug!("[Network] Player_bought_card");

        let event = NetworkEvent::BoughtCard(card, id);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_bought_card : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_sell_card(&mut self, card_index: usize, is_joker: bool) -> bool {
        debug!("[Network] Player_sell_card");

        let event = NetworkEvent::SellCard(card_index, is_joker);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_sell_card : {:?}", e);
            return false;
        }

        true
    }

    pub fn send_cash_out(&mut self, to_send: i32) -> bool {
        debug!("[Network] Player_cash_out");

        let event = NetworkEvent::Cashout(to_send);
        if let Err(e) = self.send_event_to_opponent(event) {
            error!("[Network] Player_cash_out : {:?}", e);
            return false;
        }

        true
    }

    pub fn rematch(&mut self, game_manipulation: &mut GameManipulation) -> bool {
        debug!("[Network] Network_rematch");

        if let Some(peer) = &self.opponent {
            match self.state {
                NetworkState::OpponentRematched => {
                    game_manipulation.register_event(GameManipulationEvent::OnRematch);
                    self.state = NetworkState::WaitForUserAction;
                }
                _ => {
                    game_manipulation.regenerate_seed();
                    self.state = NetworkState::WaitingForRematchResponse;
                    game_manipulation
                        .register_event(GameManipulationEvent::OnWaitingForRematchResponse);
                }
            }

            let event = NetworkEvent::Rematch(game_manipulation.get_seed());
            let packet = serialize(&event).unwrap().into_boxed_slice();

            let mut socket = get_ws().unwrap().lock().unwrap();
            let socket = socket.as_mut().unwrap();
            socket.channel_mut(0).send(packet, *peer);

            return true;
        }

        warn!("[Network] Network_rematch : No opponent found");
        false
    }

    pub fn on_messages_received(
        &mut self,
        game_manipulation: &mut GameManipulation,
        peer_id: &PeerId,
        packet: &Box<[u8]>,
    ) {
        let event: NetworkEvent = deserialize(&packet).unwrap();
        if !matches!(event, NetworkEvent::Ping() | NetworkEvent::Pong()) {
            info!("Message from {peer_id}: {event:?}");
        }

        match event {
            NetworkEvent::Ping() => {
                if let Err(e) = self.send_event_to_opponent(NetworkEvent::Pong()) {
                    error!("[Network] Received ping: {:?}", e);
                    return;
                }
            }
            NetworkEvent::Pong() => {
                self.rtt = self.ping_measurement.elapsed().as_millis() as usize;
                self.is_waiting_for_pong = false;
                debug!(
                    "[Network] Received pong with timestamp: RTT: {} ms",
                    self.rtt
                );
                game_manipulation.register_event(GameManipulationEvent::OnRTTUpdated(self.rtt));
            }
            NetworkEvent::NewPeer(peer, connected_date, seed) => match self.state {
                NetworkState::Search => {
                    info!("[Network] Opponent {peer} want to play,waiting for response...");
                    self.state = NetworkState::Found;
                    self.opponent = Some(peer_id.clone());

                    let opponent_connected_date = OffsetDateTime::parse(
                        connected_date.as_str(),
                        &time::format_description::well_known::Rfc3339,
                    )
                    .unwrap();
                    if opponent_connected_date < self.connected_date.unwrap() {
                        game_manipulation.set_seed(seed);
                    }

                    game_manipulation.register_event(GameManipulationEvent::OnRandomFound);
                }
                _ => {
                    warn!("[Network] Wrong state {:?} for NewPeer", self.state);
                }
            },
            NetworkEvent::ConfirmMatch => {
                info!("[Network] Opponent confirmed match");
                match self.state {
                    NetworkState::Found => {
                        self.state = NetworkState::OpponentConfirm;
                    }
                    NetworkState::SelfConfirm => {
                        self.state = NetworkState::Start;

                        game_manipulation
                            .register_event(GameManipulationEvent::OnAcceptedRandomMath);
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for ConfirmMatch", self.state);
                    }
                }
            }
            NetworkEvent::HighlightedCard(cards) => {
                debug!("[Network] Opponent highlighted cards: {:?}", cards);
                let current_hand = game_manipulation.get_current_hands_left();
                if current_hand == 0 {
                    debug!("[Network] No more hands left, ignoring highlighted cards and tell opponent to play");

                    if let Err(e) =
                        self.send_event_to_opponent(NetworkEvent::HighlightedCard(vec![]))
                    {
                        error!("[Network] Trying to send empty highlighted cards: {:?}", e);
                        return;
                    }

                    self.state = NetworkState::PlayTurn(cards.clone());
                    game_manipulation.register_event(GameManipulationEvent::PlayTurn(cards));
                    return;
                }

                match self.state {
                    NetworkState::WaitForUserAction => {
                        self.state = NetworkState::OpponentHighlighted(cards);
                        game_manipulation
                            .register_event(GameManipulationEvent::OpponentHighlightedCard);
                    }
                    NetworkState::SentHighlighted(_) => {
                        self.state = NetworkState::PlayTurn(cards.clone());
                        game_manipulation
                            .register_event(GameManipulationEvent::UpdateMessage("".to_string()));
                        game_manipulation.register_event(GameManipulationEvent::PlayTurn(cards));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for HighlightedCard", self.state);
                    }
                }
            }
            NetworkEvent::NewCardAlignement(alignement, _type) => {
                debug!("[Network] Opponent new cards alignement: {:?}", alignement);
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::PlayTurn(_) => {
                        game_manipulation.register_event(
                            GameManipulationEvent::NewHandCardsAlignement(
                                alignement.clone(),
                                _type,
                            ),
                        );
                    }
                    _ => {
                        warn!(
                            "[Network] Wrong state {:?} for NewCardAlignement",
                            self.state
                        );
                    }
                }
            }
            NetworkEvent::SortedHandSuit => {
                debug!("[Network] Opponent sorted hand by suit");
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::PlayTurn(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::SortedHandSuitCards);
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for SortedHandSuit", self.state);
                    }
                }
            }
            NetworkEvent::SortedHandValue => {
                debug!("[Network] Opponent sorted hand by value");
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::PlayTurn(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::SortedHandValueCards);
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for SortedHandValue", self.state);
                    }
                }
            }
            NetworkEvent::DiscardedCards(cards) => {
                debug!("[Network] Opponent discarded cards: {:?}", cards);
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::PlayTurn(_) => {
                        game_manipulation.register_event(
                            GameManipulationEvent::OpponentDiscardedHandCards(cards),
                        );
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for DiscardedCards", self.state);
                    }
                }
            }
            NetworkEvent::NewCardToEmplace(card_conf) => {
                debug!("[Network] Opponent sent new card: {:?}", card_conf);
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::PlayTurn(_)
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::EmplaceOpponentCard(card_conf));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for NewJokerCard", self.state);
                    }
                }
            }
            NetworkEvent::WaitForYourAction => {
                debug!("[Network] Opponent waiting for your action");
                match &self.state {
                    NetworkState::WaitForUserAction => {
                        self.state = NetworkState::OpponentWaitingForYou;
                    }
                    NetworkState::WaitForOpponent(game_event) => {
                        game_manipulation.acknowledge_event();

                        if let GameManipulationEvent::ProcessRemainingEvents = &(game_event.clone())
                        {
                            game_manipulation.register_event(GameManipulationEvent::UpdateMessage(
                                "Playing opponent shop choices".to_string(),
                            ));
                        } else {
                            game_manipulation.register_event(GameManipulationEvent::UpdateMessage(
                                "".to_string(),
                            ));
                        }

                        game_manipulation.register_event(game_event.clone());
                        self.state = NetworkState::WaitForUserAction;
                    }
                    _ => {
                        warn!(
                            "[Network] Wrong state {:?} for WaitForYourAction",
                            self.state
                        );
                    }
                }
            }
            NetworkEvent::UsedConsumeableCard(index, areat_type, cards_index, targets) => {
                debug!("[Network] Opponent used consumeable card: {:?}", index);
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation.register_event(
                            GameManipulationEvent::UsedConsumeableCard(
                                index,
                                areat_type,
                                cards_index,
                                targets,
                            ),
                        );
                    }
                    _ => {
                        warn!(
                            "[Network] Wrong state {:?} for UsedConsumeableCard",
                            self.state
                        );
                    }
                }
            }
            NetworkEvent::VoucherUsed(card) => {
                debug!("[Network] Opponent used voucher card: {:?}", card);
                match self.state {
                    NetworkState::WaitForOpponent(_) | NetworkState::WaitForUserAction => {
                        game_manipulation
                            .register_event(GameManipulationEvent::UsedVoucherCard(card));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for VoucherUsed", self.state);
                    }
                }
            }
            NetworkEvent::OpenBooster(card, shop_jokers_cards_conf) => {
                debug!("[Network] Opponent opened booster: {:?}", card);
                match self.state {
                    NetworkState::SentHighlighted(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation.register_event(GameManipulationEvent::OpenBooster(
                            card,
                            shop_jokers_cards_conf,
                        ));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for OpenBooster", self.state);
                    }
                }
            }
            NetworkEvent::HighlightedBoosterCard(cards) => {
                debug!("[Network] Opponent highlighted booster card: {:?}", cards);
                match self.state {
                    NetworkState::WaitForUserAction | NetworkState::WaitForOpponent(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::HighlightedBoosterCard(cards));
                    }
                    _ => {
                        warn!(
                            "[Network] Wrong state {:?} for HighlightedBoosterCard",
                            self.state
                        );
                    }
                }
            }
            NetworkEvent::RerollShop => {
                debug!("[Network] Opponent reroll shop");
                match self.state {
                    NetworkState::PlayTurn(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation.register_event(GameManipulationEvent::RerollShop);
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for RerollShop", self.state);
                    }
                }
            }
            NetworkEvent::BoughtCard(card_index, id) => {
                debug!("[Network] Opponent bought card: {:?}", card_index);
                match self.state {
                    NetworkState::PlayTurn(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::BoughtCard(card_index, id));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for BoughtCard", self.state);
                    }
                }
            }
            NetworkEvent::SellCard(card_index, is_joker) => {
                debug!("[Network] Opponent sell card: {:?}", card_index);
                match self.state {
                    NetworkState::OpponentHighlighted(_)
                    | NetworkState::SentHighlighted(_)
                    | NetworkState::PlayTurn(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::SellCard(card_index, is_joker));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for SellCard", self.state);
                    }
                }
            }
            NetworkEvent::Cashout(to_ease) => {
                debug!("[Network] Opponent cash out: {:?}", to_ease);
                match self.state {
                    NetworkState::PlayTurn(_)
                    | NetworkState::WaitForUserAction
                    | NetworkState::WaitForOpponent(_) => {
                        game_manipulation
                            .register_event(GameManipulationEvent::OpponentCashOut(to_ease));
                    }
                    _ => {
                        warn!("[Network] Wrong state {:?} for Cashout", self.state);
                    }
                }
            }
            NetworkEvent::Rematch(seed) => {
                info!("[Network] Opponent rematch request");
                match self.state {
                    NetworkState::WaitingForRematchResponse => {
                        self.state = NetworkState::WaitForUserAction;
                        game_manipulation.register_event(GameManipulationEvent::OnRematch);
                    }
                    _ => {
                        self.state = NetworkState::OpponentRematched;
                        game_manipulation.set_seed(seed);
                        game_manipulation
                            .register_event(GameManipulationEvent::OnOpponentRematched);
                    }
                }
            }
        }
    }
}

impl fmt::Display for NetworkState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
