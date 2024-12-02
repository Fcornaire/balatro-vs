use crate::{get_modules, is_ws_routine_finished, reset_modules, reset_ws};
use mlua::{lua_State, Lua};

#[derive(Clone)]
pub struct LuaPatcher {
    lua: Lua,
}

impl LuaPatcher {
    pub fn get_lua_state(&self) -> &Lua {
        &self.lua
    }

    pub unsafe fn new(lua_state: *mut lua_State) -> Self {
        let lua = Lua::init_from_ptr(lua_state);
        Self { lua }
    }

    pub fn load_chunk(&self, chunk: &str) -> mlua::Result<()> {
        self.lua.load(chunk).exec()
    }

    pub fn call_fn(&self, to_call: &str) -> mlua::Result<()> {
        let func: mlua::Function = self.lua.globals().get(to_call)?;
        func.call(())
    }

    pub fn execute_lua_function_with_result<T: mlua::FromLua>(
        &self,
        to_call: &str,
    ) -> mlua::Result<T> {
        let function: mlua::Function = self.lua.globals().get(to_call)?;

        function.call::<T>(())
    }

    pub fn execute_lua_function_with_args<T: mlua::IntoLuaMulti>(
        &self,
        fn_to_call: &str,
        args: T,
    ) -> Result<(), String> {
        let lua_args = match args.into_lua_multi(&self.lua) {
            Ok(value) => value,
            Err(e) => {
                return Err(format!("Failed to convert args to lua value: {}", e));
            }
        };

        let function: mlua::Function = self.lua.globals().get(fn_to_call).unwrap();
        function.call::<()>(lua_args).unwrap();

        Ok(())
    }

    fn register_lua_function<F, A, R>(&self, name: &str, func: F)
    where
        F: 'static + Send + Fn(&mlua::Lua, A) -> mlua::Result<R>,
        A: mlua::FromLuaMulti,
        R: mlua::IntoLuaMulti,
    {
        let lua_func = self.lua.create_function(func).unwrap();
        self.lua.globals().set(name, lua_func).unwrap();
    }

    pub fn patch_lua_state(&self) {
        self.register_lua_function("updater_get_and_update_last_version", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .updater_get_and_update_last_version()
        });

        self.register_lua_function("updater_check_for_update", |_, ()| {
            get_modules().lock().unwrap().updater_check_for_update()
        });

        self.register_lua_function("updater_update", |_, ()| {
            get_modules().lock().unwrap().updater_update()
        });

        self.register_lua_function("updater_should_update", |_, ()| {
            Ok(get_modules().lock().unwrap().updater_should_update())
        });

        self.register_lua_function("network_start_matchmaking", |_, ()| {
            get_modules().lock().unwrap().network_start_matchmaking()
        });

        self.register_lua_function("network_start_versus_friendlies", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_start_versus_friendlies()
        });

        self.register_lua_function("network_start_versus_friendlies_pairing", |_, room_code| {
            get_modules()
                .lock()
                .unwrap()
                .network_start_versus_friendlies_pairing(room_code)
        });

        self.register_lua_function("network_reset", |_, ()| {
            reset_ws();
            reset_modules();
            Ok(())
        });

        self.register_lua_function("network_is_ws_routine_finished", |_, ()| {
            Ok(is_ws_routine_finished())
        });

        self.register_lua_function("network_quit_matchmaking", |_, ()| {
            get_modules().lock().unwrap().network_quit_matchmaking()
        });

        self.register_lua_function("network_poll_and_update", |_, ()| {
            get_modules().lock().unwrap().network_poll_and_update()
        });

        self.register_lua_function("network_confirm_versus_matchmaking", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_confirm_versus_matchmaking()
        });

        self.register_lua_function("network_send_highlighted_card", |_, cards| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_highlighted_card(cards)
        });

        self.register_lua_function("network_wait_for_next_action", |_, ()| {
            get_modules().lock().unwrap().network_wait_for_next_action()
        });

        self.register_lua_function(
            "network_send_to_opponent_new_cards_alignement",
            |_, (alignement, _type)| {
                get_modules()
                    .lock()
                    .unwrap()
                    .network_send_to_opponent_new_cards_alignement(alignement, _type)
            },
        );

        self.register_lua_function("network_player_sort_hand_suit", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_player_sort_hand_suit()
        });

        self.register_lua_function("network_player_sort_hand_value", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_player_sort_hand_value()
        });

        self.register_lua_function("network_player_discarded_cards", |_, discarded| {
            get_modules()
                .lock()
                .unwrap()
                .network_player_discarded_cards(discarded)
        });

        self.register_lua_function("network_has_opponent_highlithed_cards", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_has_opponent_highlithed_cards()
        });

        self.register_lua_function("game_manipulation_acknowledge_event", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .game_manipulation_acknowledge_event()
        });

        self.register_lua_function("network_send_new_card", |_, card_conf| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_new_card(card_conf)
        });

        self.register_lua_function("network_wait_for_opponent_action_on_end_shop", |_, ()| {
            get_modules()
                .lock()
                .unwrap()
                .network_wait_for_opponent_action_on_end_shop()
        });

        self.register_lua_function(
            "network_wait_for_opponent_action_on_end_shop_after_events",
            |_, ()| {
                get_modules()
                    .lock()
                    .unwrap()
                    .network_wait_for_opponent_action_on_end_shop_after_events()
            },
        );

        self.register_lua_function(
            "network_player_use_consumeable_card",
            |_, (index, area_type, cards_index, targets)| {
                get_modules()
                    .lock()
                    .unwrap()
                    .network_player_use_consumeable_card(index, area_type, cards_index, targets)
            },
        );

        self.register_lua_function("network_player_use_voucher_card", |_, card| {
            get_modules()
                .lock()
                .unwrap()
                .network_player_use_voucher_card(card)
        });

        self.register_lua_function(
            "network_send_open_booster",
            |_, (card, shop_jokers_cards_conf)| {
                get_modules()
                    .lock()
                    .unwrap()
                    .network_send_open_booster(card, shop_jokers_cards_conf)
            },
        );

        self.register_lua_function("network_player_skip_booster", |_, ()| {
            get_modules().lock().unwrap().network_player_skip_booster()
        });

        self.register_lua_function("network_send_new_card_from_booster", |_, card_index| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_new_card_from_booster(card_index)
        });

        self.register_lua_function("network_send_reroll_shop", |_, ()| {
            get_modules().lock().unwrap().network_send_reroll_shop()
        });

        self.register_lua_function("network_send_bought_card", |_, (card, id)| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_bought_card(card, id)
        });

        self.register_lua_function("network_send_sell_card", |_, (index, is_consumeable)| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_sell_card(index, is_consumeable)
        });

        self.register_lua_function("network_send_cash_out", |_, (to_send, is_ending_shop)| {
            get_modules()
                .lock()
                .unwrap()
                .network_send_cash_out(to_send, is_ending_shop)
        });
    }
}
