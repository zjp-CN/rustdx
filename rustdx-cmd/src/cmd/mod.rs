use argh::FromArgs;
use eyre::Result;

mod day;
mod east;

pub use self::{
    day::{auto_prefix, DayCmd},
    east::EastCmd,
};

const VERSION: &str = env!("RUSTDX_VERSION");

#[derive(FromArgs, PartialEq, Debug)]
/// rustdx
pub struct TopLevel {
    #[argh(subcommand)]
    sub: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Day(DayCmd),
    EastMoney(EastCmd),
    Help(Show),
}

/// rustdx 版本号、调试
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "show")]
struct Show {
    /// 版本号。
    #[argh(switch, short = 'v')]
    version: bool,

    /// 可选。打印 TopLevel（及子命令） 结构体。比如 `rustdx -p day`。
    #[argh(switch, short = 'p', long = "print-struct")]
    pub print_struct: bool,
}

impl TopLevel {
    pub fn match_subcmd(&self) -> Result<()> {
        use SubCommand::*;
        match &self.sub {
            Day(cmd) => cmd.help_info().run(),
            EastMoney(cmd) => cmd.run(),
            Help(help) => {
                if help.version {
                    println!("当前版本号：{VERSION}");
                }
                if help.print_struct {
                    println!("{self:#?}");
                }
                Ok(())
            }
        }
    }
}
