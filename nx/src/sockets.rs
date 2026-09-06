use std::alloc::Layout;
use std::sync::Once;
use std::time::Duration;

use skyline::nn;

const POOL_SIZE: usize = 0x100000; //1mb
const ALLOC_POOL_SIZE: u64 = 0x20000; //128kb
const CONCURRENCY_LIMIT: i32 = 14;

static SOCKET_INIT: Once = Once::new();

pub fn init_sockets() {
    SOCKET_INIT.call_once(|| unsafe {
        nn::nifm::Initialize();
        nn::nifm::SubmitNetworkRequest();

        let mut spins = 0;

        while nn::nifm::IsNetworkRequestOnHold() && spins < 500 {
            std::thread::sleep(Duration::from_millis(10));
            spins += 1;
        }

        if !nn::nifm::IsNetworkAvailable() {
            println!("[bvs-nx] no network after {} ms", spins * 10);
        }

        let layout = Layout::from_size_align(POOL_SIZE, 0x1000).unwrap();
        let pool = std::alloc::alloc(layout);
        let rc = nn::socket::Initialize(pool, POOL_SIZE as u64, ALLOC_POOL_SIZE, CONCURRENCY_LIMIT);
        if rc != 0 {
            println!("[bvs-nx] nn::socket::Initialize -> {rc:#x}");
        }
    });
}
