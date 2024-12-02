use std::collections::VecDeque;

use itertools::Itertools;
use rand::distributions::{Alphanumeric, DistString};
use tracing::{debug, error, warn};

use crate::{
    get_lua_state_ptrs,
    lua_patcher::LuaPatcher,
    macros::macros::{
        call_lua_function, execute_lua_function_with_args, execute_lua_function_with_result,
    },
};

use super::{card_conf::AreaType, network::NetworkState, CardConf};

#[derive(Debug, Clone, PartialEq)]
pub enum GameManipulationEvent {
    OnRandomMatchmakingSelected,
    OnRandomFound,
    OnAcceptedRandomMath,
    OpponentDisconnectedBeforeConfirm,
    OpponentDisconnectedInGame,
    OpponentHighlightedCard,
    PlayTurn(Vec<usize>),
    NewHandCardsAlignement(Vec<usize>, String),
    SortedHandSuitCards,
    SortedHandValueCards,
    OpponentDiscardedHandCards(Vec<usize>),
    DiscardedHandCards,
    EmplaceOpponentCard(CardConf),
    ProcessRemainingEvents,
    EndShopAndStartNewRound,
    UsedConsumeableCard(usize, AreaType, Vec<usize>, Vec<CardConf>),
    UsedVoucherCard(CardConf),
    OpenBooster(CardConf, Vec<CardConf>),
    OnRTTUpdated(usize),
    HighlightedBoosterCard(Vec<usize>),
    RerollShop,
    BoughtCard(CardConf, String),
    SellCard(usize, bool),
    CashOut,
    OpponentCashOut(i32),
    WaitForOpponentAction,
    UpdateMessage(String),
}

pub struct GameManipulation {
    seed: String,
    event_queue: VecDeque<GameManipulationEvent>,
    is_ack_needed: bool,
    is_timer_ack_needed: bool,
    last_network_state: NetworkState,
}

impl GameManipulation {
    pub fn new() -> Self {
        let seed: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);

        Self {
            seed,
            is_ack_needed: false,
            is_timer_ack_needed: false,
            event_queue: VecDeque::new(),
            last_network_state: NetworkState::Idle,
        }
    }

    pub fn set_seed(&mut self, seed: String) {
        self.seed = seed;
    }

    pub fn get_seed(&self) -> String {
        self.seed.clone()
    }

    /// Register an event to be processed
    ///
    /// Some events might be processed immediately
    #[allow(unused_parens)]
    //TODO: Move event executed immediately to a separate function
    pub fn register_event(&mut self, event: GameManipulationEvent) {
        //Execute message event immediately,we don't want to be blocked by akcnowledge event
        if let GameManipulationEvent::UpdateMessage(message) = &event {
            execute_lua_function_with_args!("on_update_message", (message.clone(), String));
            return;
        }

        //Same with cash out
        if let GameManipulationEvent::OpponentCashOut(amount) = &event {
            execute_lua_function_with_args!("on_opponent_cash_out", (*amount, i32));
            return;
        }

        //Same with raw cash out
        if let GameManipulationEvent::CashOut = &event {
            execute_lua_function_with_args!(
                "on_create_timer",
                (65, usize),
                ("on_shop_action_over".to_string(), String),
                (true, bool)
            );
            return;
        }

        //Same with WaitForOpponentAction
        if let GameManipulationEvent::WaitForOpponentAction = &event {
            call_lua_function!("on_stop_timer");
            return;
        }

        //Same with OpponentDisconnectedInGame
        if let GameManipulationEvent::OpponentDisconnectedInGame = &event {
            call_lua_function!("on_opponent_disconnected_in_game");
            return;
        }

        self.event_queue.push_back(event);
    }

    pub fn get_current_hands_left(&self) -> usize {
        execute_lua_function_with_result!("get_current_hands_left", usize)
    }

    /// Waiting for long animation to finish
    /// before processing more events
    pub fn is_acknowledge_needed(&self, event: &GameManipulationEvent) -> bool {
        matches!(event, GameManipulationEvent::OpponentDiscardedHandCards(_))
            || matches!(event, GameManipulationEvent::PlayTurn(_))
            || matches!(event, GameManipulationEvent::OpenBooster(_, _))
            || matches!(event, GameManipulationEvent::EndShopAndStartNewRound)
            || matches!(event, GameManipulationEvent::HighlightedBoosterCard(_))
            || matches!(
                event,
                GameManipulationEvent::UsedConsumeableCard(_, _, _, _)
            )
            || matches!(event, GameManipulationEvent::BoughtCard(_, _))
            || matches!(event, GameManipulationEvent::RerollShop)
            || matches!(event, GameManipulationEvent::SellCard(_, _))
            || matches!(event, GameManipulationEvent::UsedVoucherCard(_))
    }

    pub fn handle_timer_with_state_update(&mut self, state: NetworkState) {
        if self.last_network_state != state {
            match state {
                NetworkState::WaitForUserAction => {
                    debug!("[GameManipulation] Start a new timer for waiting user action");
                    execute_lua_function_with_args!(
                        "on_create_timer",
                        (45, usize),
                        ("on_wait_for_user_action_over".to_string(), String)
                    );
                }
                NetworkState::SentHighlighted(_) | NetworkState::PlayTurn(_) => {
                    debug!("[GameManipulation] Stop the current timer");
                    call_lua_function!("on_stop_timer");
                }
                _ => {}
            }

            self.last_network_state = state;
        }
    }

    #[allow(unused_parens)]
    //TODO: Remove unhandled events when those are processed right away
    pub fn process_events(&mut self) {
        if self.is_ack_needed {
            return;
        }

        if let Some(event) = self.event_queue.pop_front() {
            if self.is_acknowledge_needed(&event) {
                self.is_ack_needed = true;
            }

            match event {
                GameManipulationEvent::OnRandomMatchmakingSelected => {
                    call_lua_function!("on_random_search");
                }
                GameManipulationEvent::OnRandomFound => {
                    call_lua_function!("on_random_found");
                }
                GameManipulationEvent::OnAcceptedRandomMath => {
                    execute_lua_function_with_args!("on_random_start", (self.get_seed(), String));
                }
                GameManipulationEvent::OpponentDisconnectedBeforeConfirm => {
                    call_lua_function!("on_opponent_disconnected_from_found");
                }
                GameManipulationEvent::OpponentDisconnectedInGame => {
                    //This event is handled immediately at register; nothing to do here
                }
                GameManipulationEvent::PlayTurn(cards) => {
                    execute_lua_function_with_args!("on_play_turn", (cards, Vec<usize>));
                }
                GameManipulationEvent::OpponentHighlightedCard => {
                    call_lua_function!("on_opponent_highlighted_card")
                }
                GameManipulationEvent::NewHandCardsAlignement(new_alignement, _type) => {
                    let mut to_treat: VecDeque<Vec<usize>> = VecDeque::new();
                    to_treat.push_front(new_alignement);
                    while let Some(event) = self.event_queue.pop_front() {
                        if let GameManipulationEvent::NewHandCardsAlignement(alignement, _type) =
                            event
                        {
                            to_treat.push_back(alignement);
                        } else {
                            self.event_queue.push_front(event);
                            break;
                        }
                    }

                    let cards_update =
                        self.get_cards_alignements_update(&mut to_treat, _type.clone());
                    execute_lua_function_with_args!(
                        "on_opponent_new_card_alignement",
                        (cards_update, Vec<usize>),
                        (_type, String)
                    );
                }
                GameManipulationEvent::SortedHandSuitCards => {
                    call_lua_function!("on_opponent_sort_hand_suit");
                }
                GameManipulationEvent::SortedHandValueCards => {
                    call_lua_function!("on_opponent_sort_hand_value");
                }
                GameManipulationEvent::OpponentDiscardedHandCards(to_discard) => {
                    execute_lua_function_with_args!(
                        "on_opponent_discarded_cards",
                        (to_discard, Vec<usize>)
                    );
                }
                GameManipulationEvent::DiscardedHandCards => {
                    self.is_timer_ack_needed = true;
                    call_lua_function!("on_pause_timer"); //Pause timer while discarding cards
                }
                GameManipulationEvent::EmplaceOpponentCard(conf) => {
                    execute_lua_function_with_args!("on_opponent_new_card", (conf, CardConf));
                }
                GameManipulationEvent::EndShopAndStartNewRound => {
                    execute_lua_function_with_args!("on_update_message", ("".to_string(), String)); //Clear message
                    call_lua_function!("on_end_shop");
                }
                GameManipulationEvent::UsedConsumeableCard(index, area_type, cards, targets) => {
                    let is_consumeable = area_type == AreaType::Consumeables;

                    execute_lua_function_with_args!(
                        "on_opponent_use_consumeable_card",
                        (index, usize),
                        (is_consumeable, bool),
                        (cards, Vec<usize>),
                        (targets, Vec<CardConf>)
                    );
                }
                GameManipulationEvent::UsedVoucherCard(conf) => {
                    execute_lua_function_with_args!(
                        "on_opponent_use_voucher_card",
                        (conf, CardConf)
                    );
                }
                GameManipulationEvent::OnRTTUpdated(rtt) => {
                    execute_lua_function_with_args!("on_rtt_updated", (rtt, usize));
                }
                GameManipulationEvent::OpenBooster(conf, shop_jokers_cards_conf) => {
                    execute_lua_function_with_args!(
                        "on_opponent_open_booster",
                        (conf, CardConf),
                        (shop_jokers_cards_conf, Vec<CardConf>)
                    );
                }
                GameManipulationEvent::HighlightedBoosterCard(cards) => {
                    execute_lua_function_with_args!(
                        "on_highlighted_booster_card",
                        (cards, Vec<usize>)
                    );
                }
                GameManipulationEvent::ProcessRemainingEvents => {
                    call_lua_function!("on_processed_remaining_events");
                }
                GameManipulationEvent::UpdateMessage(_) => {
                    //This event is handled immediately at register; nothing to do here
                }
                GameManipulationEvent::RerollShop => {
                    call_lua_function!("on_opponent_reroll_shop");
                }
                GameManipulationEvent::BoughtCard(card, id) => {
                    execute_lua_function_with_args!(
                        "on_opponent_bought_card",
                        (card, CardConf),
                        (id, String)
                    );
                }
                GameManipulationEvent::SellCard(index, is_consumeable) => {
                    execute_lua_function_with_args!(
                        "on_opponent_sell_card",
                        (index, usize),
                        (is_consumeable, bool)
                    );
                }
                GameManipulationEvent::WaitForOpponentAction => {
                    //This event is handled immediately at register; nothing to do here
                }
                GameManipulationEvent::CashOut => {
                    //This event is handled immediately at register; nothing to do here
                }
                GameManipulationEvent::OpponentCashOut(_) => {
                    //This event is handled immediately at register; nothing to do here
                }
            }
        }
    }

    pub fn acknowledge_event(&mut self) {
        if !self.is_ack_needed {
            warn!("[GameManipulation] Acknowledge called but no ack needed");
            return;
        }

        self.is_ack_needed = false;

        if self.is_timer_ack_needed {
            call_lua_function!("on_resume_timer");
            self.is_timer_ack_needed = false;
        }
    }

    fn get_cards_alignements_update(
        &mut self,
        to_process: &mut VecDeque<Vec<usize>>,
        _type: String,
    ) -> Vec<usize> {
        let opponent_length = match _type.as_str() {
            "hand" => execute_lua_function_with_result!("get_opponent_hand_length", usize),
            "jokers" => execute_lua_function_with_result!("get_opponent_jokers_length", usize),
            "consumeables" => {
                execute_lua_function_with_result!("get_opponent_consumeables_length", usize)
            }
            _ => panic!("Unknown type: {}", _type),
        };

        let mut cards_update: Vec<usize> = (1..opponent_length + 1).collect_vec();

        to_process.drain(..).for_each(|alignement| {
            alignement.windows(2).for_each(|window| {
                if let [a, b] = window {
                    cards_update.swap(*a - 1, *b - 1);
                }
            });
        });

        cards_update
    }
}
