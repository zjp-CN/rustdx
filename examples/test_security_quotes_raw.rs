#!/usr/bin/env rustx
/**
调试版：查看SecurityQuotes的原始响应数据
*/
use rustdx_complete::tcp::{Tcp, Tdx};
use rustdx_complete::tcp::stock::SecurityQuotes;
use std::net::SocketAddr;

fn main() {
    println!("🔍 调试SecurityQuotes原始数据\n");

    let addr: SocketAddr = "115.238.56.198:7709".parse().unwrap();
    match Tcp::new_with_ip(&addr) {
        Ok(mut tcp) => {
            println!("✅ 连接成功\n");

            // 测试普通股票
            println!("1️⃣  测试普通股票(000001平安银行)...");
            let mut quotes = SecurityQuotes::new(vec![(0, "000001")]);

            // 先发送请求
            println!("   发送请求包: {:02x?}", quotes.send());
            println!("   请求包长度: {} 字节", quotes.send().len());

            match quotes.recv(&mut tcp) {
                Ok(response) => {
                    println!("   ✅ 响应成功");
                    println!("   响应包大小: {} 字节", response.len());
                    println!("   响应数据(前64字节): {:02x?}", &response[..response.len().min(64)]);
                }
                Err(e) => {
                    println!("   ❌ 失败: {}", e);
                    println!("   错误详情: {:?}\n", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 连接失败: {}", e);
        }
    }
}
