mod api;
mod config;
mod error;
mod snapshot;
mod sync;

use crate::config::Config;
use std::process;
fn main() {
    let cfg = Config::from_args();

    if let Err(err) = snapshot::run(&cfg) {
        eprintln!("  {err}");
        process::exit(1);
    }
}
