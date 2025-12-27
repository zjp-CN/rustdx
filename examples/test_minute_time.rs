#!/usr/bin/env rustx
/**
测试MinuteTime功能，获取股票分时数据
*/
use rustdx_complete::tcp::{Tcp, Tdx};
use rustdx_complete::tcp::stock::MinuteTime;

fn main() {
    println!("🚀 测试MinuteTime功能\n");

    // 创建TCP连接，尝试多个服务器
    println!("1️⃣  连接到通达信服务器...");

    // 首先尝试默认连接
    match Tcp::new() {
        Ok(mut tcp) => {
            println!("   ✅ 连接成功\n");
            test_minute_time(&mut tcp);
        }
        Err(e) => {
            println!("   ❌ 默认连接失败: {}，尝试其他服务器...", e);

            // 尝试其他服务器IP
            use rustdx_complete::tcp::ip::STOCK_IP;
            let mut last_error = e.to_string();
            let mut connected = false;

            for (i, ip) in STOCK_IP.iter().enumerate().take(5) {
                println!("\n   尝试服务器 #{}: {}...", i + 1, ip);
                match Tcp::new_with_ip(ip) {
                    Ok(mut tcp) => {
                        println!("   ✅ 连接成功\n");
                        test_minute_time(&mut tcp);
                        connected = true;
                        break;
                    }
                    Err(e) => {
                        last_error = format!("{} (服务器#{})", e, i + 1);
                        println!("   ❌ 失败: {}", e);
                    }
                }
            }

            if !connected {
                println!("\n   ❌ 所有服务器连接失败");
                println!("   最后错误: {}\n", last_error);
                return;
            }
        }
    }

    println!("\n✅ 测试完成！");
}

fn test_minute_time(tcp: &mut Tcp) {
    // 测试深市股票分时数据
    println!("2️⃣  测试获取000001平安银行的分时数据...");
    let mut minute = MinuteTime::new(0, "000001");

    match minute.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");
            println!("   📊 返回数量: {} 个数据点\n", minute.result().len());

            if minute.result().len() > 0 {
                println!("   前10个数据点:");
                for (i, data) in minute.result().iter().take(10).enumerate() {
                    println!("      {:2}. 价格: {:>7.2}  成交量: {}", i + 1, data.price, data.vol);
                }

                println!("\n   最后5个数据点:");
                let len = minute.result().len();
                for (i, data) in minute.result().iter().skip(len - 5).enumerate() {
                    println!("      {:2}. 价格: {:>7.2}  成交量: {}", len - 5 + i + 1, data.price, data.vol);
                }

                // 计算简单的统计数据
                let prices: Vec<f64> = minute.result().iter().map(|d| d.price).collect();
                let max_price = prices.iter().fold(0.0f64, |a, &b| a.max(b));
                let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let first_price = prices[0];
                let last_price = prices[prices.len() - 1];

                println!("\n   📈 统计信息:");
                println!("      开盘价: {:.2}", first_price);
                println!("      最高价: {:.2}", max_price);
                println!("      最低价: {:.2}", min_price);
                println!("      最新价: {:.2}", last_price);
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }

    // 测试沪市股票分时数据
    println!("\n3️⃣  测试获取600000浦发银行的分时数据...");
    let mut minute = MinuteTime::new(1, "600000");

    match minute.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");
            println!("   📊 返回数量: {} 个数据点\n", minute.result().len());

            if minute.result().len() > 0 {
                println!("   前10个数据点:");
                for (i, data) in minute.result().iter().take(10).enumerate() {
                    println!("      {:2}. 价格: {:>7.2}  成交量: {}", i + 1, data.price, data.vol);
                }

                // 计算简单的统计数据
                let prices: Vec<f64> = minute.result().iter().map(|d| d.price).collect();
                let max_price = prices.iter().fold(0.0f64, |a, &b| a.max(b));
                let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));

                println!("\n   📈 统计信息:");
                println!("      最高价: {:.2}", max_price);
                println!("      最低价: {:.2}", min_price);
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }
}
