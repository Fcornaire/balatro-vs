use card_conf::{AreaType, CardConf};
use game_manipulation::{GameManipulation, GameManipulationEvent};
use mlua::{Result, Table};
use network::Network;
use tracing::{error, info};
use updater::Updater;

use crate::{get_modules, get_runtime};

pub mod bvs;
pub mod card_conf;
pub mod game_manipulation;
pub mod integrity;
pub mod network;
pub mod updater;

pub struct Modules {
    network: Network,
    game_manipulation: GameManipulation,
    updater: Updater,
}

impl Modules {
    pub fn init() -> Self {
        info!("[Modules] Initializing modules...");
        let mut updater = Updater::new();
        updater.update_current_version();

        Self {
            network: Network::new(),
            game_manipulation: GameManipulation::new(),
            updater,
        }
    }

    pub fn updater_update_current_version(&mut self) -> Result<()> {
        self.updater.update_current_version();
        Ok(())
    }

    pub fn updater_get_and_update_last_version(&mut self) -> Result<()> {
        let url = self.updater.get_repository_url().to_string();

        get_runtime().spawn(async move {
            match Updater::get_last_stable_version(&url).await {
                Ok(res) => {
                    let updater = &mut get_modules().lock().unwrap().updater;
                    updater.set_last_version(res.clone());
                }
                Err(e) => error!("[Modules] Failed to fetch the latest version: {:?}", e),
            }
        });

        Ok(())
    }

    pub fn updater_should_update(&mut self) -> Result<bool> {
        Ok(self.updater.should_update())
    }

    pub fn updater_check_for_update(&mut self) -> Result<bool> {
        if self.updater.is_updating() {
            return Ok(false);
        }

        let current_version = self.updater.get_current_version();
        let last_version = self.updater.get_last_version();

        if current_version == "0.0.0" || last_version == "0.0.0" {
            return Ok(false);
        }

        if current_version != last_version && !self.updater.is_updating() {
            info!(
                "[Modules] New version available: {} (current: {})",
                last_version, current_version
            );

            let url = self.updater.get_base_download_url().to_string();
            let last_version_clone = last_version.clone();
            self.updater.set_is_updating(true);

            get_runtime().spawn(async move {
                let is_success = Updater::trigger_update(&url, &last_version_clone).await;
                if is_success {
                    let updater = &mut get_modules().lock().unwrap().updater;
                    updater.set_should_update(true);
                }
            });

            return Ok(false);
        }

        Ok(current_version == last_version)
    }

    pub fn updater_update(&mut self) -> Result<()> {
        self.updater.update();

        Ok(())
    }

    pub fn network_poll_and_update(&mut self) -> Result<()> {
        self.network.poll_and_update(&mut self.game_manipulation);
        Ok(())
    }

    pub fn network_start_matchmaking(&mut self) -> Result<bool> {
        let res = self.network.start_matchmaking(&mut self.game_manipulation);
        Ok(res)
    }

    pub fn network_start_versus_friendlies(&mut self) -> Result<String> {
        let res = self.network.start_versus_friendlies();

        if res.is_empty() {
            error!("[Modules] Failed to start versus friendlies");
            return Err(mlua::Error::RuntimeError(
                "Failed to start versus friendlies".to_string(),
            ));
        }

        Ok(res)
    }

    pub fn network_start_versus_friendlies_pairing(&mut self, room_code: String) -> Result<bool> {
        let res = self.network.start_versus_friendlies_pairing(room_code);
        Ok(res)
    }

    pub fn network_quit_matchmaking(&mut self) -> Result<bool> {
        let res = self.network.quit_server();
        Ok(res)
    }

    pub fn network_confirm_versus_matchmaking(&mut self) -> Result<bool> {
        let res = self
            .network
            .confirm_versus_matchmaking(&mut self.game_manipulation);

        if !res {
            error!("[Modules] Failed to confirm versus matchmaking");
            return Err(mlua::Error::RuntimeError(
                "Failed to confirm versus matchmaking".to_string(),
            ));
        }

        Ok(res)
    }

    pub fn network_send_highlighted_card(&mut self, highlighted_cards: Table) -> Result<bool> {
        let cards: Vec<usize> = highlighted_cards
            .pairs::<usize, usize>()
            .map(|pair| pair.unwrap())
            .map(|(_, card)| card)
            .collect();
        let res = self
            .network
            .send_highlighted_card(&mut self.game_manipulation, cards);

        if !res {
            error!("[Modules] Failed to send highlighted card");
            return Err(mlua::Error::RuntimeError(
                "Failed to send highlighted card".to_string(),
            ));
        }

        Ok(res)
    }

    pub fn network_wait_for_next_action(&mut self) -> Result<()> {
        self.network.wait_for_next_action();
        Ok(())
    }

    pub fn network_send_to_opponent_new_cards_alignement(
        &mut self,
        alignement: Table,
        _type: String,
    ) -> Result<()> {
        let alignement: Vec<usize> = alignement
            .pairs::<usize, usize>()
            .map(|pair| pair.unwrap())
            .map(|(_, card_index)| card_index)
            .collect();
        self.network
            .send_to_opponent_new_cards_alignement(alignement, _type);
        Ok(())
    }

    pub fn network_player_sort_hand_suit(&mut self) -> Result<()> {
        self.network.player_sort_hand_suit();
        Ok(())
    }

    pub fn network_player_sort_hand_value(&mut self) -> Result<()> {
        self.network.player_sort_hand_value();
        Ok(())
    }

    pub fn network_player_discarded_cards(&mut self, discarded_cards: Table) -> Result<()> {
        let discarded_cards: Vec<usize> = discarded_cards
            .pairs::<usize, usize>()
            .map(|pair| pair.unwrap())
            .map(|(_, card_index)| card_index)
            .collect();

        self.network
            .player_discarded_cards(discarded_cards, &mut self.game_manipulation);
        Ok(())
    }

    pub fn network_has_opponent_highlithed_cards(&mut self) -> Result<bool> {
        Ok(self.network.has_opponent_highlithed_cards())
    }

    pub fn network_send_new_card(&mut self, table: Table) -> Result<()> {
        let conf = CardConf::from(table);

        self.network.send_new_card(conf);
        Ok(())
    }

    pub fn game_manipulation_acknowledge_event(&mut self) -> Result<()> {
        self.game_manipulation.acknowledge_event();
        Ok(())
    }

    pub fn network_wait_for_opponent_action_on_end_shop(&mut self) -> Result<()> {
        self.network.wait_for_opponent_action_before(
            &mut self.game_manipulation,
            GameManipulationEvent::ProcessRemainingEvents,
        );

        self.game_manipulation
            .register_event(GameManipulationEvent::WaitForOpponentAction);

        Ok(())
    }

    pub fn network_wait_for_opponent_action_on_end_shop_after_events(&mut self) -> Result<()> {
        self.network.wait_for_opponent_action_before(
            &mut self.game_manipulation,
            GameManipulationEvent::EndShopAndStartNewRound,
        );
        Ok(())
    }

    pub fn network_player_use_consumeable_card(
        &mut self,
        index: usize,
        area_type: String,
        cards_index: Table,
        targets: Table,
    ) -> Result<()> {
        let cards_index: Vec<usize> = cards_index
            .pairs::<usize, usize>()
            .map(|pair| pair.unwrap())
            .map(|(_, card_index)| card_index)
            .collect();

        let targets: Vec<CardConf> = targets
            .pairs::<usize, Table>()
            .map(|pair| pair.unwrap())
            .map(|(_, target_table)| CardConf::from(target_table))
            .collect();

        let area = match area_type.as_str() {
            "consumeables" => AreaType::Consumeables,
            "pack_cards" => AreaType::PackCards,
            _ => panic!("Unknown area type: {}", area_type),
        };

        let res = self
            .network
            .player_use_consumeable_card(index, area, cards_index, targets);

        if !res {
            error!("[Modules] Failed to use consumeable card");
            return Err(mlua::Error::RuntimeError(
                "Failed to use consumeable card".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_player_use_voucher_card(&mut self, card: Table) -> Result<()> {
        let card = CardConf::from(card);
        let res = self.network.player_use_voucher_card(card);

        if !res {
            error!("[Modules] Failed to use voucher card");
            return Err(mlua::Error::RuntimeError(
                "Failed to use voucher card".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_open_booster(
        &mut self,
        card: Table,
        shop_jokers_cards_conf: Table,
    ) -> Result<()> {
        let card = CardConf::from(card);
        let shop_jokers_cards_conf: Vec<CardConf> = shop_jokers_cards_conf
            .pairs::<usize, Table>()
            .map(|pair| pair.unwrap())
            .map(|(_, card_table)| CardConf::from(card_table))
            .collect();

        let res = self.network.send_open_booster(card, shop_jokers_cards_conf);

        if !res {
            error!("[Modules] Failed to send open booster");
            return Err(mlua::Error::RuntimeError(
                "Failed to send open booster".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_player_skip_booster(&mut self) -> Result<()> {
        let res = self.network.player_skip_booster();

        if !res {
            error!("[Modules] Failed to skip booster");
            return Err(mlua::Error::RuntimeError(
                "Failed to skip booster".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_new_card_from_booster(&mut self, card_index: usize) -> Result<()> {
        let res = self.network.send_new_card_from_booster(card_index);

        if !res {
            error!("[Modules] Failed to send new card from booster");
            return Err(mlua::Error::RuntimeError(
                "Failed to send new card from booster".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_reroll_shop(&mut self) -> Result<()> {
        let res = self.network.send_reroll_shop();

        if !res {
            error!("[Modules] Failed to send reroll shop");
            return Err(mlua::Error::RuntimeError(
                "Failed to send reroll shop".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_bought_card(&mut self, card: Table, id: String) -> Result<()> {
        let card = CardConf::from(card);
        let res = self.network.send_bought_card(card, id);

        if !res {
            error!("[Modules] Failed to send bought card");
            return Err(mlua::Error::RuntimeError(
                "Failed to send bought card".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_sell_card(&mut self, index: usize, is_consumeable: bool) -> Result<()> {
        let res = self.network.send_sell_card(index, is_consumeable);

        if !res {
            error!("[Modules] Failed to send sell card");
            return Err(mlua::Error::RuntimeError(
                "Failed to send sell card".to_string(),
            ));
        }

        Ok(())
    }

    pub fn network_send_cash_out(&mut self, to_send: i32, is_ending_shop: bool) -> Result<()> {
        let res = self.network.send_cash_out(to_send);

        if !res {
            error!("[Modules] Failed to send cash out");
            return Err(mlua::Error::RuntimeError(
                "Failed to send cash out".to_string(),
            ));
        }

        if !is_ending_shop {
            self.game_manipulation
                .register_event(GameManipulationEvent::CashOut);
        }

        Ok(())
    }

    pub fn network_rematch(&mut self) -> Result<()> {
        let res = self.network.rematch(&mut self.game_manipulation);

        if !res {
            error!("[Modules] Failed to send rematch request");
            return Err(mlua::Error::RuntimeError(
                "Failed to send rematch request".to_string(),
            ));
        }

        Ok(())
    }
}
