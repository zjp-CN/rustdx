#!/usr/bin/env rustx
/**
测试SecurityList功能，获取股票列表
*/
use rustdx_complete::tcp::{Tcp, Tdx};
use rustdx_complete::tcp::stock::SecurityList;

fn main() {
    println!("🚀 测试SecurityList功能\n");

    // 创建TCP连接，尝试多个服务器
    println!("1️⃣  连接到通达信服务器...");

    // 首先尝试默认连接
    match Tcp::new() {
        Ok(mut tcp) => {
            println!("   ✅ 连接成功\n");
            test_list(&mut tcp);
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
                        test_list(&mut tcp);
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

fn test_list(tcp: &mut Tcp) {
    // 测试深市股票列表
    println!("2️⃣  测试获取深市股票列表 (market=0, start=0)...");
    let mut list = SecurityList::new(0, 0);

    match list.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");
            println!("   📊 返回数量: {} 只股票\n", list.result().len());

            println!("   前10只股票:");
            for (i, stock) in list.result().iter().take(10).enumerate() {
                println!("      {:2}. {} {} - 成交量单位:{}",
                    i + 1,
                    stock.code,
                    stock.name,
                    stock.volunit
                );
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }

    // 测试分页获取
    println!("\n3️⃣  测试分页获取深市股票 (market=0, start=1000)...");
    let mut list = SecurityList::new(0, 1000);

    match list.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");
            println!("   📊 返回数量: {} 只股票\n", list.result().len());

            if list.result().len() > 0 {
                println!("   前10只股票:");
                for (i, stock) in list.result().iter().take(10).enumerate() {
                    println!("      {:2}. {} {} - 成交量单位:{}",
                        i + 1,
                        stock.code,
                        stock.name,
                        stock.volunit
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }

    // 测试沪市股票列表
    println!("\n4️⃣  测试获取沪市股票列表 (market=1, start=0)...");
    let mut list = SecurityList::new(1, 0);

    match list.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");
            println!("   📊 返回数量: {} 只股票\n", list.result().len());

            if list.result().len() > 0 {
                println!("   前10只股票:");
                for (i, stock) in list.result().iter().take(10).enumerate() {
                    println!("      {:2}. {} {} - 成交量单位:{}",
                        i + 1,
                        stock.code,
                        stock.name,
                        stock.volunit
                    );
                }
            } else {
                println!("   ⚠️  沪市数据为空（服务器可能不支持）");
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }
}
