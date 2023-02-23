mod cmd;
mod io;

#[macro_use]
extern crate log;

use eyre::Result;

fn main() -> Result<()> {
    env_logger::init();
    let cmd: cmd::TopLevel = argh::from_env();
    cmd.match_subcmd()
}
