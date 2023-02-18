use rustdx::tcp::{self, Tcp, Tdx};
use std::io::Result;

fn main() -> Result<()> {
    log_init()?;
    for addr in tcp::ip::STOCK_IP.iter() {
        if let Ok(mut tcp) = Tcp::new_with_ip(addr) {
            println!("{:21?} Connected to the server!", addr);

            let mut day = tcp::stock::Kline::default();
            println!("#000001# 最近三天 K 线\n{:#?}", day.recv_parsed(&mut tcp)?);

            println!(
                "list len: {}",
                tcp::SecurityList::new(0, 0).recv_parsed(&mut tcp)?.len()
            );

            break;
        } else {
            panic!("{:21?} Couldn't connect to server...", addr);
        }
    }
    Ok(())
}

fn log_init() -> Result<()> {
    use simplelog::{Config, LevelFilter, WriteLogger};
    use std::fs::File;

    let mut unrecognized_log_level = None;
    #[rustfmt::skip]
    let log_level = match std::env::var("LOG") {
        Err(_) => LevelFilter::Error,
        Ok(s)  => match s.as_ref() {
            "off"     | "OFF"     => LevelFilter::Off,
            "error"   | "ERROR"   => LevelFilter::Error,
            "warning" | "WARNING" => LevelFilter::Warn,
            "info"    | "INFO"    => LevelFilter::Info,
            "debug"   | "DEBUG"   => LevelFilter::Debug,
            "trace"   | "TRACE"   => LevelFilter::Trace,
            _ => {
                unrecognized_log_level = Some(s);
                LevelFilter::Error
            }
        },
    };

    let _ = WriteLogger::init(log_level, Config::default(), File::create("rustdx.log")?);
    if let Some(unrecognized) = unrecognized_log_level {
        log::error!(
            "`{}` is an unrecognized log level. Falling back to log level 'error'. \
                     Recognized log levels are: 'off', 'error', 'warning', 'info', 'debug' and \
                     'trace'.",
            unrecognized
        );
    }
    Ok(())
}
