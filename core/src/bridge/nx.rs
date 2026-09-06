//! [`LuaBridge`] over raw Lua 5.1 C API function pointers supplied by the host

use core::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::OnceLock;

use crate::bridge::{self, LuaBridge, BRIDGE_LOAD, LUA_BRIDGE, LUA_BRIDGE_NAME, NATIVE_DISPATCH};

pub type LuaState = c_void;
pub type LuaCFunction = unsafe extern "C" fn(*mut LuaState) -> c_int;

const LUA_GLOBALSINDEX: c_int = -10002;
const LUA_TNIL: c_int = 0;
const LUA_TNUMBER: c_int = 3;
const LUA_TSTRING: c_int = 4;
const LUA_TFUNCTION: c_int = 6;

/// Lua 5.1 C API entries the bridge needs
#[derive(Clone, Copy)]
pub struct LuaApi {
    pub loadbuffer: unsafe extern "C" fn(*mut LuaState, *const u8, usize, *const c_char) -> c_int,
    pub pcall: unsafe extern "C" fn(*mut LuaState, c_int, c_int, c_int) -> c_int,
    pub error: unsafe extern "C" fn(*mut LuaState) -> c_int,
    pub gettop: unsafe extern "C" fn(*mut LuaState) -> c_int,
    pub settop: unsafe extern "C" fn(*mut LuaState, c_int),
    pub getfield: unsafe extern "C" fn(*mut LuaState, c_int, *const c_char),
    pub setfield: unsafe extern "C" fn(*mut LuaState, c_int, *const c_char),
    pub type_: unsafe extern "C" fn(*mut LuaState, c_int) -> c_int,
    pub tolstring: unsafe extern "C" fn(*mut LuaState, c_int, *mut usize) -> *const c_char,
    pub checklstring: unsafe extern "C" fn(*mut LuaState, c_int, *mut usize) -> *const c_char,
    pub pushlstring: unsafe extern "C" fn(*mut LuaState, *const u8, usize),
    pub pushcclosure: unsafe extern "C" fn(*mut LuaState, LuaCFunction, c_int),
}

static API: OnceLock<LuaApi> = OnceLock::new();

fn api() -> &'static LuaApi {
    API.get().expect("bridge::nx::install was not called")
}

unsafe fn string_at(l: *mut LuaState, idx: c_int) -> String {
    let mut len = 0usize;
    let lstring = (api().tolstring)(l, idx, &mut len);
    if lstring.is_null() {
        String::new()
    } else {
        String::from_utf8_lossy(core::slice::from_raw_parts(lstring as *const u8, len)).into_owned()
    }
}

unsafe fn error_at_top(l: *mut LuaState) -> String {
    let s = string_at(l, -1);
    if s.is_empty() {
        "unknown error".to_string()
    } else {
        s
    }
}

unsafe fn exec_chunk(l: *mut LuaState, chunk: &str, chunk_name: &str) -> Result<(), String> {
    let a = api();
    let top = (a.gettop)(l);
    let cname = format!("{chunk_name}\0");
    if (a.loadbuffer)(
        l,
        chunk.as_ptr(),
        chunk.len(),
        cname.as_ptr() as *const c_char,
    ) != 0
    {
        let err = error_at_top(l);
        (a.settop)(l, top);
        return Err(format!("compile error: {err}"));
    }

    if (a.pcall)(l, 0, 0, 0) != 0 {
        let err = error_at_top(l);
        (a.settop)(l, top);
        return Err(err);
    }

    (a.settop)(l, top);
    Ok(())
}

pub struct NxBridge {
    state: AtomicPtr<LuaState>,
}

impl LuaBridge for NxBridge {
    fn exec(&self, chunk: &str, chunk_name: &str) -> Result<(), String> {
        unsafe { exec_chunk(self.state.load(Ordering::Relaxed), chunk, chunk_name) }
    }

    fn call(&self, name: &str, args: &[&str]) -> Result<String, String> {
        unsafe {
            let api = api();
            let lua_state = self.state.load(Ordering::Relaxed);
            let top = (api.gettop)(lua_state);
            let cname = format!("{name}\0");
            (api.getfield)(lua_state, LUA_GLOBALSINDEX, cname.as_ptr() as *const c_char);

            if (api.type_)(lua_state, -1) != LUA_TFUNCTION {
                (api.settop)(lua_state, top);
                return Err(format!("no Lua function named {name}"));
            }

            for arg in args {
                (api.pushlstring)(lua_state, arg.as_ptr(), arg.len());
            }

            if (api.pcall)(lua_state, args.len() as c_int, 1, 0) != 0 {
                let err = error_at_top(lua_state);
                (api.settop)(lua_state, top);
                return Err(err);
            }

            let result = match (api.type_)(lua_state, -1) {
                LUA_TSTRING | LUA_TNUMBER => string_at(lua_state, -1),
                _ => String::new(),
            };

            (api.settop)(lua_state, top);
            Ok(result)
        }
    }
}

unsafe fn arg_string(l: *mut LuaState, idx: c_int) -> String {
    let mut len = 0usize;
    let lstring = (api().checklstring)(l, idx, &mut len);
    String::from_utf8_lossy(core::slice::from_raw_parts(lstring as *const u8, len)).into_owned()
}

unsafe extern "C" fn native_dispatch(l: *mut LuaState) -> c_int {
    let a = api();
    let name = arg_string(l, 1);
    let args = if (a.type_)(l, 2) == LUA_TNIL {
        "[]".to_string()
    } else {
        arg_string(l, 2)
    };

    let reply = bridge::dispatch(&name, &args);
    (a.pushlstring)(l, reply.as_ptr(), reply.len());
    1
}

unsafe extern "C" fn bridge_load(l: *mut LuaState) -> c_int {
    if let Err(e) = exec_chunk(l, LUA_BRIDGE, LUA_BRIDGE_NAME) {
        let msg = format!("{BRIDGE_LOAD}: {e}");
        (api().pushlstring)(l, msg.as_ptr(), msg.len());
        return (api().error)(l);
    }
    0
}

/// Registers the native dispatcher and the bridge loader on `state`
/// and makes it the target of native -> Lua calls
pub unsafe fn install(state: *mut LuaState, api: LuaApi) {
    let _ = API.set(api);
    let lua_api = API.get().unwrap();
    let top = (lua_api.gettop)(state);
    for (name, f) in [
        (NATIVE_DISPATCH, native_dispatch as LuaCFunction),
        (BRIDGE_LOAD, bridge_load as LuaCFunction),
    ] {
        let cname = format!("{name}\0");
        (lua_api.pushcclosure)(state, f, 0);
        (lua_api.setfield)(state, LUA_GLOBALSINDEX, cname.as_ptr() as *const c_char);
    }

    (lua_api.settop)(state, top);

    if bridge::bridge().is_none() {
        bridge::set_bridge(Box::new(NxBridge {
            state: AtomicPtr::new(state),
        }));
    }
}
