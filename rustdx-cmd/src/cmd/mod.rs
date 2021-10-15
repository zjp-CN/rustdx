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

    /// 可选。打印 TopLevel（及子命令） 结构体。比如 `rustdx -p day`。
    #[argh(switch, short = 'p', long = "print-struct")]
    pub print_struct: bool,
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
        if self.print_struct {
            println!("{:#?}", self);
        }
        match self.sub {
            Day(ref cmd) => cmd.help_info().run(),
            Official(ref cmd) => cmd.run(),
            EastMoney(ref cmd) => cmd.run(),
        }
    }
}
