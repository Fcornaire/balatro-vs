pub mod bridge;
pub mod macros;
pub mod modules;
pub mod transport;

use modules::bvs::BvsConfig;
use modules::Modules;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};
use transport::Transport;

pub use bridge::lua_print;

static THREAD_START_HOOK: OnceCell<fn()> = OnceCell::new();

pub fn set_thread_start_hook(hook: fn()) {
    let _ = THREAD_START_HOOK.set(hook);
}

pub(crate) fn run_thread_start_hook() {
    if let Some(hook) = THREAD_START_HOOK.get() {
        hook();
    }
}

pub fn get_bvs_config() -> &'static Arc<BvsConfig> {
    static INSTANCE: OnceCell<Arc<BvsConfig>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        #[cfg(target_os = "windows")]
        {
            use std::fs;
            use std::path::PathBuf;
            if let Ok(app_data_dir) = std::env::var("APPDATA") {
                let mod_folder = PathBuf::from(app_data_dir)
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
            panic!("[bvs_config] Could not find app data directory");
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Android and Switch, the Lua side knows where the mod folder is
            use crate::macros::macros::execute_lua_function_with_result;

            let bvs_conf = execute_lua_function_with_result!("get_bvs_json", String);
            if bvs_conf.is_empty() {
                panic!("[bvs_config] Config file not found (bvs.json missing from mod folder)");
            }
            match serde_json::from_str::<BvsConfig>(&bvs_conf) {
                Ok(config) => return Arc::new(config),
                Err(e) => {
                    panic!("[bvs_config] Failed to parse config: {:?}", e);
                }
            }
        }
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

/// The connection to the opponent (WebRTC socket or relay client)
static TRANSPORT: OnceCell<Arc<Mutex<Option<Transport>>>> = OnceCell::new();

/// True once a transport exists and its background routine has ended
pub fn is_ws_routine_finished() -> bool {
    match TRANSPORT.get() {
        Some(transport) => match transport.lock() {
            Ok(guard) => guard
                .as_ref()
                .map(|t| !t.is_routine_running())
                .unwrap_or(false),
            Err(_) => false,
        },
        None => false,
    }
}

#[cfg(not(target_os = "switch"))]
static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

#[cfg(not(target_os = "switch"))]
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

pub fn get_transport() -> Option<&'static Arc<Mutex<Option<Transport>>>> {
    TRANSPORT.get()
}

pub fn set_transport(transport: Transport) {
    if TRANSPORT.get().is_some() && TRANSPORT.get().unwrap().lock().unwrap().is_some() {
        warn!("[Network] transport already set");
        return;
    }

    if TRANSPORT.get().is_none() {
        match TRANSPORT.set(Arc::new(Mutex::new(Some(transport)))) {
            Ok(_) => debug!("[Network] transport set"),
            Err(_) => error!("[Network] failed to set the transport"),
        }
    } else {
        TRANSPORT.get().unwrap().lock().unwrap().replace(transport);
        debug!("[Network] transport replaced");
    }
}

pub fn reset_transport() {
    if TRANSPORT.get().is_none() {
        warn!("[Network] transport already reset");
        return;
    }

    if let Some(transport) = TRANSPORT.get() {
        match transport.lock() {
            Ok(mut transport) => {
                transport.take();
            }
            Err(poison) => {
                let mut transport = poison.into_inner();
                transport.take();
            }
        }
    }

    debug!("[Network] transport reset");
}

#[cfg(all(target_os = "windows", feature = "mlua"))]
const MAX_LOG_FILES: usize = 10;

#[cfg(all(target_os = "windows", feature = "mlua"))]
fn create_log_file() -> Option<std::fs::File> {
    use std::{
        fs::{create_dir_all, read_dir},
        path::PathBuf,
    };

    use time::OffsetDateTime;

    let dir = PathBuf::from(std::env::var("APPDATA").ok()?)
        .join("Balatro")
        .join("Mods")
        .join("balatro-vs")
        .join("logs");
    create_dir_all(&dir).ok()?;

    let mut logs: Vec<PathBuf> = read_dir(&dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| {
                    name.starts_with("bvs-") && name.ends_with(".log")
                })
        })
        .collect();
    logs.sort();
    let excess = logs.len().saturating_sub(MAX_LOG_FILES - 1);
    for path in logs.iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }

    let now = OffsetDateTime::now_utc();
    let stamp = format!(
        "{:04}.{:02}.{:02}-{:02}.{:02}.{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );

    let mut path = dir.join(format!("bvs-{stamp}.log"));
    if path.exists() {
        path = dir.join(format!("bvs-{stamp}-{}.log", std::process::id()));
    }

    std::fs::File::create(path).ok()
}

#[cfg(all(target_os = "windows", feature = "mlua"))]
fn init_tracing_with_file() {
    use tracing_subscriber::prelude::*;

    let console = tracing_subscriber::fmt::layer()
        .compact()
        .with_thread_names(true)
        .with_target(true)
        .with_ansi(false); // lovely console doesn't support ANSI

    let file_layer = create_log_file().map(|f| {
        tracing_subscriber::fmt::layer()
            .compact()
            .with_thread_names(true)
            .with_target(true)
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(f))
    });

    let file_filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing::Level::INFO)
        .with_target("winmm", tracing::Level::DEBUG)
        .with_target("matchbox_socket", tracing::Level::DEBUG);

    let _ = tracing_subscriber::registry()
        .with(console.with_filter(tracing_subscriber::filter::LevelFilter::INFO))
        .with(file_layer.map(|l| l.with_filter(file_filter)))
        .try_init();
}

/// Windows `winmm.dll` proxy, hooks `luaL_newstate` and installs the
/// bridge into every state the game creates
#[cfg(all(target_os = "windows", feature = "mlua"))]
mod windows_symbols {
    use super::*;
    use mlua::lua_State;
    use retour::static_detour;
    use std::ffi::c_void;
    use windows::core::{s, w};
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    static_detour! {
        pub static LuaLNewState_Detour: unsafe extern "C" fn() -> *mut lua_State;
    }

    unsafe extern "C" fn lua_newstatex_detour() -> *mut lua_State {
        debug!("New lua state to hook!");

        let state = LuaLNewState_Detour.call();
        bridge::mlua::install_state(state);

        state
    }

    #[no_mangle]
    #[allow(non_snake_case)]
    unsafe extern "system" fn DllMain(_: HINSTANCE, reason: u32, _: *const c_void) -> u8 {
        if reason != 1 {
            return 1;
        }

        init_tracing_with_file();

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
}

/// Android / Linux, loaded from Lua with `package.loadlib`
#[cfg(all(any(target_os = "linux", target_os = "android"), feature = "mlua"))]
mod linux_symbols {
    use super::*;
    use mlua::lua_State;

    #[no_mangle]
    pub unsafe extern "C" fn luaopen_winmm(lua: *mut lua_State) -> i32 {
        tracing_subscriber::fmt()
            .compact()
            .with_max_level(tracing::Level::DEBUG)
            .with_ansi(false)
            .init();

        bridge::mlua::install_state(lua);

        0
    }
}
