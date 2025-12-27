#!/usr/bin/env rustx
/**
测试FinanceInfo功能，获取股票财务信息
*/
use rustdx_complete::tcp::{Tcp, Tdx};
use rustdx_complete::tcp::stock::FinanceInfo;

fn main() {
    println!("🚀 测试FinanceInfo功能\n");

    // 创建TCP连接，尝试多个服务器
    println!("1️⃣  连接到通达信服务器...");

    // 首先尝试默认连接
    match Tcp::new() {
        Ok(mut tcp) => {
            println!("   ✅ 连接成功\n");
            test_finance_info(&mut tcp);
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
                        test_finance_info(&mut tcp);
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

fn test_finance_info(tcp: &mut Tcp) {
    // 测试深市股票财务信息
    println!("2️⃣  测试获取000001平安银行的财务信息...");
    let mut finance = FinanceInfo::new(0, "000001");

    match finance.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");

            if finance.result().len() > 0 {
                let info = &finance.result()[0];
                println!("   📊 基本信息:");
                println!("      股票代码: {}", info.code);
                println!("      市场代码: {}", info.market);
                println!("      上市日期: {}", info.ipo_date);
                println!("      更新日期: {}", info.updated_date);

                println!("\n   💰 股本信息:");
                println!("      总股本: {:.0} 股", info.zongguben);
                println!("      流通股本: {:.0} 股", info.liutongguben);

                println!("\n   📈 财务指标:");
                println!("      总资产: {:.0} 元", info.zongzichan);
                println!("      流动资产: {:.0} 元", info.liudongzichan);
                println!("      固定资产: {:.0} 元", info.gudingzichan);
                println!("      净资产: {:.0} 元", info.jingzichan);

                println!("\n   💹 利润表:");
                println!("      主营收入: {:.0} 元", info.zhuyingshouru);
                println!("      营业利润: {:.0} 元", info.yingyelirun);
                println!("      净利润: {:.0} 元", info.jinglirun);

                println!("\n   📊 现金流:");
                println!("      经营现金流: {:.0} 元", info.jingyingxianjinliu);
                println!("      总现金流: {:.0} 元", info.zongxianjinliu);
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }

    // 测试沪市股票财务信息
    println!("\n3️⃣  测试获取600000浦发银行的财务信息...");
    let mut finance = FinanceInfo::new(1, "600000");

    match finance.recv_parsed(tcp) {
        Ok(_) => {
            println!("   ✅ 获取成功\n");

            if finance.result().len() > 0 {
                let info = &finance.result()[0];
                println!("   📊 基本信息:");
                println!("      股票代码: {}", info.code);
                println!("      上市日期: {}", info.ipo_date);

                println!("\n   💰 股本信息:");
                println!("      总股本: {:.0} 股", info.zongguben);
                println!("      流通股本: {:.0} 股", info.liutongguben);

                println!("\n   📈 财务指标:");
                println!("      总资产: {:.0} 元", info.zongzichan);
                println!("      净资产: {:.0} 元", info.jingzichan);

                println!("\n   💹 利润表:");
                println!("      主营收入: {:.0} 元", info.zhuyingshouru);
                println!("      净利润: {:.0} 元", info.jinglirun);
            }
        }
        Err(e) => {
            println!("   ❌ 获取失败: {}\n", e);
        }
    }
}
