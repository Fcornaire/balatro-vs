//! Attaches the core to the Lua state received from lovely-injector-nx

use core::ffi::c_void;
use std::sync::atomic::Ordering;

use bvs_core::bridge::nx::{self as nx_bridge, LuaApi as BridgeApi};
use lovely_nx_api::LuaApi;
use skyline::nn::os::{
    ChangeThreadPriority, GetCurrentThread, GetThreadAvailableCoreMask, GetThreadPriority,
};

fn fill_random(dest: &mut [u8]) {
    unsafe { skyline::nn::os::GenerateRandomBytes(dest.as_mut_ptr() as *mut _, dest.len() as u64) }
}

fn getrandom_v02(dest: &mut [u8]) -> Result<(), getrandom02::Error> {
    fill_random(dest);
    Ok(())
}

getrandom02::register_custom_getrandom!(getrandom_v02);

struct LogWriter(Vec<u8>);

impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        while let Some(nl) = self.0.iter().position(|b| *b == b'\n') {
            let line = String::from_utf8_lossy(&self.0[..nl]).into_owned();
            println!("[bvs] {}", line.trim_end());
            self.0.drain(..=nl);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if !self.0.is_empty() {
            println!("[bvs] {}", String::from_utf8_lossy(&self.0));
            self.0.clear();
        }
        Ok(())
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(false)
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(|| LogWriter(Vec::new()))
        .try_init();
}

extern "C" {
    #[link_name = "\u{1}_ZN2nn2os17SetThreadCoreMaskEPNS0_10ThreadTypeEim"]
    fn SetThreadCoreMask(
        thread: *mut skyline::nn::os::ThreadType,
        ideal_core: i32,
        affinity_mask: u64,
    );
    #[link_name = "\u{1}_ZN2nn2os20GetCurrentCoreNumberEv"]
    fn GetCurrentCoreNumber() -> i32;
}

static MAIN_CORE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// Keeps a worker thread off the game's main core
fn worker_thread_setup() {
    unsafe {
        let thread = GetCurrentThread();
        let main_core = MAIN_CORE.load(Ordering::Relaxed);
        let mask = GetThreadAvailableCoreMask();
        let other = (0..64u32).find(|c| mask & (1u64 << c) != 0 && *c as i32 != main_core);
        match other {
            Some(core) => SetThreadCoreMask(thread, core as i32, 1u64 << core),
            None => {
                let prio = GetThreadPriority(thread);
                ChangeThreadPriority(thread, prio + 2);
            }
        }
    }
}

fn bridge_api(api: &LuaApi) -> BridgeApi {
    BridgeApi {
        loadbuffer: api.loadbuffer,
        pcall: api.pcall,
        error: api.error,
        gettop: api.gettop,
        settop: api.settop,
        getfield: api.getfield,
        setfield: api.setfield,
        type_: api.type_,
        tolstring: api.tolstring,
        checklstring: api.checklstring,
        pushlstring: api.pushlstring,
        pushcclosure: api.pushcclosure,
    }
}

/// lovely-injector-nx callback
pub unsafe extern "C" fn on_ready(state: *mut c_void, api: *const LuaApi) {
    let api = &*api;
    if api.version != lovely_nx_api::API_VERSION {
        println!(
            "[bvs-nx] lovely-injector-nx API version {} != {}, not attaching",
            api.version,
            lovely_nx_api::API_VERSION
        );
        return;
    }
    init_tracing();
    crate::sockets::init_sockets();

    MAIN_CORE.store(GetCurrentCoreNumber(), Ordering::Relaxed);
    bvs_core::set_thread_start_hook(worker_thread_setup);
    bvs_core::transport::relay::set_tls_connector(crate::tls::relay_connect);
    bvs_core::modules::updater::nx::set_http_get(crate::http::get);

    nx_bridge::install(state, bridge_api(api));
    println!("[bvs-nx] core attached");
}
