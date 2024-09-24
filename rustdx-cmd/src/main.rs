mod cmd;
mod io;

#[macro_use]
extern crate log;

use eyre::Result;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cmd: cmd::TopLevel = argh::from_env();
    cmd.match_subcmd()
}
