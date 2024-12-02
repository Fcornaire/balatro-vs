pub mod lua_patcher;
pub mod macros;
pub mod modules;

use lua_patcher::LuaPatcher;
use matchbox_socket::WebRtcSocket;
use modules::bvs::BvsConfig;
use modules::Modules;
use once_cell::sync::OnceCell;
use retour::static_detour;
use std::ffi::c_void;
use std::fs;
use std::sync::atomic::AtomicPtr;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};
use windows::core::{s, w};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

use mlua::lua_State;

pub fn get_bvs_config() -> &'static Arc<BvsConfig> {
    static INSTANCE: OnceCell<Arc<BvsConfig>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        if let Some(app_data_dir) = dirs::data_dir() {
            let mod_folder = app_data_dir
                .join("Balatro")
                .join("Mods")
                .join("balatro-vs")
                .join("lovely");

            let config_path = mod_folder.join("bvs.json");
            if let Ok(config_content) = fs::read_to_string(config_path) {
                match serde_json::from_str::<BvsConfig>(&config_content) {
                    Ok(config) => return Arc::new(config),
                    Err(e) => {
                        panic!("[bvs_config] Failed to parse config: {:?}", e);
                    }
                }
            } else {
                panic!("[bvs_config] Could not read config file");
            }
        }

        panic!("[bvs_config] Could not find app data directory")
    })
}

pub fn get_modules() -> &'static Arc<Mutex<Modules>> {
    static INSTANCE: OnceCell<Arc<Mutex<Modules>>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let modules = Arc::new(Mutex::new(Modules::init()));
        modules
    })
}

pub fn reset_modules() {
    let modules = get_modules();

    *modules.lock().unwrap() = Modules::init();
}

static WEBRTCSOCKET: OnceCell<Arc<Mutex<Option<WebRtcSocket>>>> = OnceCell::new();
static LUA_STATE_PTRS: OnceCell<Arc<Mutex<Vec<AtomicPtr<lua_State>>>>> = OnceCell::new();
static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

//TODO: Find a better way to handle this, don't like relying on static mut
static mut WEBRTCSOCKET_ROUTINE_HANDLE: OnceCell<Arc<Mutex<Option<std::thread::JoinHandle<()>>>>> =
    OnceCell::new();

#[allow(static_mut_refs)]
pub fn init_or_reset_ws_routine_handle(handle: std::thread::JoinHandle<()>) {
    unsafe {
        if let Some(_) = WEBRTCSOCKET_ROUTINE_HANDLE.get() {
            reset_ws_routine_handle();
        }

        let handle = Arc::new(Mutex::new(Some(handle)));
        match WEBRTCSOCKET_ROUTINE_HANDLE.set(handle) {
            Ok(_) => {}
            Err(e) => error!("[Network] Failed to set WS routine handle: {:#?}", e),
        }
    }
}

#[allow(static_mut_refs)]
pub fn is_ws_routine_finished() -> bool {
    unsafe {
        if let Some(handle) = WEBRTCSOCKET_ROUTINE_HANDLE.get() {
            let handle_lock = handle.lock().unwrap();
            let res = handle_lock.as_ref();
            if let Some(_handle) = res {
                _handle.is_finished()
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[allow(static_mut_refs)]
pub fn reset_ws_routine_handle() {
    unsafe {
        WEBRTCSOCKET_ROUTINE_HANDLE.take();
    }
}

pub fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();
        runtime
    })
}

pub fn get_ws() -> Option<&'static Arc<Mutex<Option<WebRtcSocket>>>> {
    WEBRTCSOCKET.get()
}

pub fn get_lua_state_ptrs() -> Option<&'static Arc<Mutex<Vec<AtomicPtr<lua_State>>>>> {
    LUA_STATE_PTRS.get()
}

pub fn add_lua_state_ptr(ptr: AtomicPtr<lua_State>) {
    let lua_state_ptrs = LUA_STATE_PTRS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    lua_state_ptrs.lock().unwrap().push(ptr);
}

pub fn set_ws(socket: WebRtcSocket) {
    if WEBRTCSOCKET.get().is_some() && WEBRTCSOCKET.get().unwrap().lock().unwrap().is_some() {
        warn!("[Network] WS Socket already set");
        return;
    }

    if WEBRTCSOCKET.get().is_none() {
        match WEBRTCSOCKET.set(Arc::new(Mutex::new(Some(socket)))) {
            Ok(_) => debug!("[Network] WS Socket setted"),
            Err(_) => error!("[Network] Failed to set WS socket"),
        }
    } else {
        WEBRTCSOCKET.get().unwrap().lock().unwrap().replace(socket);
        debug!("[Network] WS Socket replaced");
    }
}

pub fn reset_ws() {
    if WEBRTCSOCKET.get().is_none() {
        warn!("[Network] WS Socket already reset");
        return;
    }

    if let Some(socket) = WEBRTCSOCKET.get() {
        match socket.lock() {
            Ok(mut socket) => {
                socket.take();
            }
            Err(poison) => {
                let mut socket = poison.into_inner();
                socket.take();
            }
        }
    }

    reset_ws_routine_handle();

    debug!("[Network] WS Socket reset");
}

static_detour! {
    pub static LuaLNewState_Detour: unsafe extern "C" fn() -> *mut lua_State;
}

unsafe extern "C" fn lua_newstatex_detour() -> *mut lua_State {
    debug!("New lua state to hook!");

    let state = LuaLNewState_Detour.call();
    add_lua_state_ptr(AtomicPtr::new(state));

    let patcher = LuaPatcher::new(state);
    patcher.patch_lua_state();

    state
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, reason: u32, _: *const c_void) -> u8 {
    if reason != 1 {
        return 1;
    }

    tracing_subscriber::fmt()
        .compact()
        .with_thread_names(true)
        .with_target(true)
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false) // lovely console doesn't support ANSI ¯\_(ツ)_/¯
        .init();

    let handle = LoadLibraryW(w!("lua51.dll")).unwrap();
    let proc_newstate = GetProcAddress(handle, s!("luaL_newstate")).unwrap();
    let fn_target_newstate =
        std::mem::transmute::<_, unsafe extern "C" fn() -> *mut lua_State>(proc_newstate);

    LuaLNewState_Detour
        .initialize(fn_target_newstate, || lua_newstatex_detour())
        .unwrap()
        .enable()
        .unwrap();

    1
}
