// Platform bridge between the game's Lua state and the core.

#[cfg(feature = "mlua")]
pub mod mlua;
#[cfg(feature = "nx")]
pub mod nx;

use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::error;

use crate::modules::card_conf::LuaCardConf;
use crate::modules::CardConf;

pub const LUA_BRIDGE: &str = include_str!("lua_bridge.lua");
pub const LUA_BRIDGE_NAME: &str = "=[balatro-vs lua_bridge.lua]";
pub const NATIVE_DISPATCH: &str = "bvs_native_dispatch";
pub const LUA_DISPATCH: &str = "bvs_lua_dispatch";
pub const BRIDGE_LOAD: &str = "bvs_bridge_load";

pub type DispatchFn = fn(name: &str, args_json: &str) -> String;

pub trait LuaBridge: Send + Sync {
    /// Run a chunk of Lua source in the game's main state
    fn exec(&self, chunk: &str, chunk_name: &str) -> Result<(), String>;
    /// Call the global Lua function `name` with string arguments
    fn call(&self, name: &str, args: &[&str]) -> Result<String, String>;
}

static BRIDGE: OnceLock<Box<dyn LuaBridge>> = OnceLock::new();

/// Install the bridge for the game's main Lua state
pub fn set_bridge(bridge: Box<dyn LuaBridge>) {
    if BRIDGE.set(bridge).is_err() {
        error!("[Bridge] bridge already installed");
    }
}

pub fn bridge() -> Option<&'static dyn LuaBridge> {
    BRIDGE.get().map(|b| b.as_ref())
}

/// Names of every function the bridge must expose as a Lua global
pub const NATIVE_FUNCTIONS: &[&str] = &[
    "updater_get_and_update_last_version",
    "updater_check_for_update",
    "updater_update",
    "updater_should_update",
    "updater_is_thunderstore_build",
    "updater_get_last_version",
    "network_start_matchmaking",
    "network_start_versus_friendlies",
    "network_start_versus_friendlies_pairing",
    "network_reset",
    "network_is_ws_routine_finished",
    "network_quit_matchmaking",
    "network_poll_and_update",
    "network_confirm_versus_matchmaking",
    "network_send_highlighted_card",
    "network_wait_for_next_action",
    "network_send_to_opponent_new_cards_alignement",
    "network_player_sort_hand_suit",
    "network_player_sort_hand_value",
    "network_player_discarded_cards",
    "network_has_opponent_highlithed_cards",
    "game_manipulation_acknowledge_event",
    "game_manipulation_start_player_shop",
    "network_send_new_card",
    "network_wait_for_opponent_action_on_end_shop",
    "network_wait_for_opponent_action_on_end_shop_after_events",
    "network_player_use_consumeable_card",
    "network_player_use_voucher_card",
    "network_send_open_booster",
    "network_player_skip_booster",
    "network_send_new_card_from_booster",
    "network_send_reroll_shop",
    "network_send_bought_card",
    "network_send_sell_card",
    "network_send_cash_out",
    "network_rematch",
];

struct Args(Vec<Value>);

impl Args {
    fn get<T: DeserializeOwned>(&self, i: usize) -> Result<T, String> {
        let v = self.0.get(i).cloned().unwrap_or(Value::Null);
        serde_json::from_value(v).map_err(|e| format!("argument #{}: {e}", i + 1))
    }

    fn opt<T: DeserializeOwned>(&self, i: usize) -> Result<Option<T>, String> {
        match self.0.get(i) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => serde_json::from_value(v.clone())
                .map(Some)
                .map_err(|e| format!("argument #{}: {e}", i + 1)),
        }
    }

    fn card(&self, i: usize) -> Result<CardConf, String> {
        self.get::<LuaCardConf>(i)?.try_into()
    }

    fn opt_card(&self, i: usize) -> Result<Option<CardConf>, String> {
        self.opt::<LuaCardConf>(i)?
            .map(TryInto::try_into)
            .transpose()
    }

    fn cards(&self, i: usize) -> Result<Vec<CardConf>, String> {
        self.get::<Vec<LuaCardConf>>(i)?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}

fn none<T>(_: T) -> Vec<Value> {
    vec![]
}

fn one<T: serde::Serialize>(v: T) -> Vec<Value> {
    vec![json!(v)]
}

/// The native entry point.
pub fn dispatch(name: &str, args_json: &str) -> String {
    if name == "__functions" {
        return NATIVE_FUNCTIONS.join(",");
    }

    if !NATIVE_FUNCTIONS.contains(&name) {
        return json!({ "error": format!("unknown native function '{name}'") }).to_string();
    }

    let args = match serde_json::from_str::<Vec<Value>>(args_json) {
        Ok(a) => Args(a),
        Err(e) => return json!({ "error": format!("{name}: bad arguments: {e}") }).to_string(),
    };

    match dispatch_inner(name, &args) {
        Ok(results) => json!({ "ok": results }).to_string(),
        Err(e) => {
            error!("[Bridge] {name} failed: {e}");
            json!({ "error": e }).to_string()
        }
    }
}

fn dispatch_inner(name: &str, a: &Args) -> Result<Vec<Value>, String> {
    let modules = crate::get_modules();
    let mut guard = modules
        .lock()
        .map_err(|e| format!("modules mutex poisoned: {e}"))?;

    match name {
        "updater_get_and_update_last_version" => {
            guard.updater_get_and_update_last_version().map(none)
        }
        "updater_check_for_update" => guard.updater_check_for_update().map(one),
        "updater_update" => guard.updater_update().map(none),
        "updater_should_update" => guard.updater_should_update().map(one),
        "updater_is_thunderstore_build" => guard.updater_is_thunderstore_build().map(one),
        "updater_get_last_version" => guard.updater_get_last_version().map(one),
        "network_start_matchmaking" => guard.network_start_matchmaking().map(one),
        "network_start_versus_friendlies" => guard.network_start_versus_friendlies().map(one),
        "network_start_versus_friendlies_pairing" => guard
            .network_start_versus_friendlies_pairing(a.get(0)?)
            .map(one),
        "network_reset" => {
            drop(guard);
            crate::reset_transport();
            crate::reset_modules();
            Ok(vec![])
        }
        "network_is_ws_routine_finished" => Ok(one(crate::is_ws_routine_finished())),
        "network_quit_matchmaking" => guard.network_quit_matchmaking().map(one),
        "network_poll_and_update" => guard.network_poll_and_update().map(none),
        "network_confirm_versus_matchmaking" => guard.network_confirm_versus_matchmaking().map(one),
        "network_send_highlighted_card" => guard.network_send_highlighted_card(a.get(0)?).map(one),
        "network_wait_for_next_action" => guard.network_wait_for_next_action().map(none),
        "network_send_to_opponent_new_cards_alignement" => guard
            .network_send_to_opponent_new_cards_alignement(a.get(0)?, a.get(1)?)
            .map(none),
        "network_player_sort_hand_suit" => guard.network_player_sort_hand_suit().map(none),
        "network_player_sort_hand_value" => guard.network_player_sort_hand_value().map(none),
        "network_player_discarded_cards" => {
            guard.network_player_discarded_cards(a.get(0)?).map(none)
        }
        "network_has_opponent_highlithed_cards" => {
            guard.network_has_opponent_highlithed_cards().map(one)
        }
        "game_manipulation_acknowledge_event" => {
            guard.game_manipulation_acknowledge_event().map(none)
        }
        "game_manipulation_start_player_shop" => {
            guard.game_manipulation_start_player_shop().map(none)
        }
        "network_send_new_card" => guard.network_send_new_card(a.card(0)?).map(none),
        "network_wait_for_opponent_action_on_end_shop" => guard
            .network_wait_for_opponent_action_on_end_shop()
            .map(none),
        "network_wait_for_opponent_action_on_end_shop_after_events" => guard
            .network_wait_for_opponent_action_on_end_shop_after_events()
            .map(none),
        "network_player_use_consumeable_card" => guard
            .network_player_use_consumeable_card(a.get(0)?, a.get(1)?, a.get(2)?, a.cards(3)?)
            .map(none),
        "network_player_use_voucher_card" => {
            guard.network_player_use_voucher_card(a.card(0)?).map(none)
        }
        "network_send_open_booster" => guard
            .network_send_open_booster(a.card(0)?, a.cards(1)?)
            .map(none),
        "network_player_skip_booster" => guard.network_player_skip_booster().map(none),
        "network_send_new_card_from_booster" => guard
            .network_send_new_card_from_booster(a.get(0)?, a.opt_card(1)?)
            .map(none),
        "network_send_reroll_shop" => guard.network_send_reroll_shop().map(none),
        "network_send_bought_card" => guard
            .network_send_bought_card(a.card(0)?, a.get(1)?)
            .map(none),
        "network_send_sell_card" => guard.network_send_sell_card(a.get(0)?, a.get(1)?).map(none),
        "network_send_cash_out" => guard.network_send_cash_out(a.get(0)?, a.get(1)?).map(none),
        "network_rematch" => guard.network_rematch().map(none),
        _ => Err(format!("unknown native function '{name}'")),
    }
}

pub trait ToLua {
    fn to_lua(&self) -> Value;
}

macro_rules! to_lua_json {
    ($($t:ty),*) => { $( impl ToLua for $t { fn to_lua(&self) -> Value { json!(self) } } )* };
}
to_lua_json!(String, &str, bool, i32, i64, u32, u64, usize, f64);

impl<T: ToLua> ToLua for Vec<T> {
    fn to_lua(&self) -> Value {
        Value::Array(self.iter().map(ToLua::to_lua).collect())
    }
}

impl<T: ToLua> ToLua for Option<T> {
    fn to_lua(&self) -> Value {
        self.as_ref().map_or(Value::Null, ToLua::to_lua)
    }
}

impl ToLua for CardConf {
    fn to_lua(&self) -> Value {
        json!(LuaCardConf::from(self))
    }
}

/// Call the global Lua function `name` through the bridge dispatcher
pub fn call_lua(name: &str, args: Vec<Value>) -> Result<Vec<Value>, String> {
    let bridge = bridge().ok_or_else(|| "Lua bridge not installed".to_string())?;
    let args_json = Value::Array(args).to_string();
    let res = bridge.call(LUA_DISPATCH, &[name, &args_json])?;
    let res: Value =
        serde_json::from_str(&res).map_err(|e| format!("{name}: bad dispatcher reply: {e}"))?;
    if let Some(err) = res.get("error") {
        return Err(err.as_str().unwrap_or("unknown error").to_string());
    }
    Ok(res
        .get("ok")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub fn call_lua_logged(name: &str, args: Vec<Value>) {
    if let Err(e) = call_lua(name, args) {
        error!("[Bridge] Failed to call lua fn {name}: {e}");
    }
}

/// Call a Lua function
pub fn call_lua_result<T: DeserializeOwned + Default>(name: &str) -> T {
    match call_lua(name, vec![]) {
        Ok(mut results) => {
            let first = if results.is_empty() {
                Value::Null
            } else {
                results.remove(0)
            };
            serde_json::from_value(first).unwrap_or_else(|e| {
                error!("[Bridge] {name}: cannot decode result: {e}");
                T::default()
            })
        }
        Err(e) => {
            error!("[Bridge] Failed to call lua fn {name}: {e}");
            T::default()
        }
    }
}

pub fn lua_print(msg: &str) {
    if let Some(bridge) = bridge() {
        if let Err(e) = bridge.call("print", &[msg]) {
            error!("[Bridge] print failed: {e}");
        }
    }
}
