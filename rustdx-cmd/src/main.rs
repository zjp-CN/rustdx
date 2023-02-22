mod cmd;
mod io;

use eyre::Result;

fn main() -> Result<()> {
    // log_init()?;
    let cmd: cmd::TopLevel = argh::from_env();
    cmd.match_subcmd()
}

// fn log_init() -> Result<()> {
//     use simplelog::{Config, LevelFilter, WriteLogger};
//     use std::fs::File;
//     let _ = WriteLogger::init(LevelFilter::Info, Config::default(),
// File::create("rustdx.log")?);     Ok(())
// }
