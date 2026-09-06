//! [`LuaBridge`] implementation on top of mlua, for the platforms where the game exports the Lua C API

use std::sync::atomic::{AtomicPtr, Ordering};

use ::mlua::{lua_State, Function, Lua};
use tracing::error;

use crate::bridge::{self, LuaBridge, BRIDGE_LOAD, LUA_BRIDGE, LUA_BRIDGE_NAME, NATIVE_DISPATCH};

pub struct MluaBridge {
    state: AtomicPtr<lua_State>,
}

impl MluaBridge {
    unsafe fn lua(&self) -> Lua {
        Lua::init_from_ptr(self.state.load(Ordering::Relaxed))
    }
}

impl LuaBridge for MluaBridge {
    fn exec(&self, chunk: &str, chunk_name: &str) -> Result<(), String> {
        let lua = unsafe { self.lua() };
        lua.load(chunk)
            .set_name(chunk_name)
            .exec()
            .map_err(|e| e.to_string())
    }

    fn call(&self, name: &str, args: &[&str]) -> Result<String, String> {
        let lua = unsafe { self.lua() };
        let func: Function = lua.globals().get(name).map_err(|e| e.to_string())?;
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let res: Option<String> = func
            .call(::mlua::MultiValue::from_vec(
                args.into_iter()
                    .map(|s| ::mlua::Value::String(lua.create_string(s).unwrap()))
                    .collect(),
            ))
            .map_err(|e| e.to_string())?;
        Ok(res.unwrap_or_default())
    }
}

/// Registers the native dispatcher and the bridge loader on `state`
pub unsafe fn install_state(state: *mut lua_State) {
    let lua = Lua::init_from_ptr(state);
    let dispatch = lua
        .create_function(|_, (name, args): (String, String)| Ok(bridge::dispatch(&name, &args)))
        .expect("create bvs_native_dispatch");
    if let Err(e) = lua.globals().set(NATIVE_DISPATCH, dispatch) {
        error!("[Bridge] cannot register {NATIVE_DISPATCH}: {e}");
        return;
    }

    let load = lua
        .create_function(|lua, ()| {
            lua.load(LUA_BRIDGE).set_name(LUA_BRIDGE_NAME).exec()?;
            Ok(())
        })
        .expect("create bvs_bridge_load");
    if let Err(e) = lua.globals().set(BRIDGE_LOAD, load) {
        error!("[Bridge] cannot register {BRIDGE_LOAD}: {e}");
        return;
    }

    if bridge::bridge().is_none() {
        bridge::set_bridge(Box::new(MluaBridge {
            state: AtomicPtr::new(state),
        }));
    }
}
