use anyhow::Result;
use argh::FromArgs;

mod day;
pub use day::*;

mod east;
pub use east::*;

mod official;
pub use official::*;

#[derive(FromArgs, PartialEq, Debug)]
/// rustdx
pub struct TopLevel {
    #[argh(subcommand)]
    sub: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommand {
    Day(DayCmd),
    Official(Official),
    EastMoney(East),
}

impl TopLevel {
    pub fn match_subcmd(&self) -> Result<()> {
        use SubCommand::*;
        match self.sub {
            Day(ref cmd) => cmd.help_info().run(),
            Official(ref cmd) => cmd.run(),
            EastMoney(ref cmd) => cmd.run(),
        }
    }
}
