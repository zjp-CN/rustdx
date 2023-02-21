use rustdx::tcp::{self, Tcp, Tdx};
use std::io::Result;

fn main() -> Result<()> {
    for addr in tcp::ip::STOCK_IP.iter() {
        if let Ok(mut tcp) = Tcp::new_with_ip(addr) {
            println!("{addr:21?} Connected to the server!");

            let mut day = tcp::stock::Kline::default();
            println!("#000001# 最近三天 K 线\n{:#?}", day.recv_parsed(&mut tcp)?);

            println!(
                "list len: {}",
                tcp::SecurityList::new(0, 0).recv_parsed(&mut tcp)?.len()
            );

            break;
        } else {
            panic!("{addr:21?} Couldn't connect to server...");
        }
    }
    Ok(())
}
