#![allow(clippy::missing_safety_doc)]

mod bvs;
mod http;
mod sockets;
mod tls;

#[skyline::main(name = "balatro_vs_nx")]
pub fn main() {
    println!("[bvs-nx] plugin loaded (v{})", env!("CARGO_PKG_VERSION"));
    match lovely_nx_api::on_ready(bvs::on_ready) {
        Ok(()) => println!("[bvs-nx] waiting for lovely-injector-nx to hand over the Lua state"),
        Err(e) => {
            println!("[bvs-nx] {e}: balatro-vs will not run, install liblovely_injector_nx.nro")
        }
    }
}
