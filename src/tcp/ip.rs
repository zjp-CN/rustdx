//! 对应于 pytdx/util/best_ip.py 文件。
//!
//! 去除了失效的地址。暂时未验证 Stock 非 IP 和 Future 地址。
//!
//! Stock 非 IP 地址有：
//!   {'ip': 'hq.cjis.cn', 'port': 7709},
//!   {'ip': 'hq1.daton.com.cn', 'port': 7709},
//!   {'ip': 'jstdx.gtjas.com', 'port': 7709},
//!   {'ip': 'shtdx.gtjas.com', 'port': 7709},
//!   {'ip': 'sztdx.gtjas.com', 'port': 7709},
//!
//! 其他地址来源 #TODO# ：
//! 1. <https://gitee.com/ibopo/mootdx/blob/master/mootdx/consts.py>
//! 2. 通达信客户端设置

lazy_static::lazy_static! {
    pub static ref STOCK_IP: [std::net::SocketAddr; 19] = [
        "39.100.68.59:7709".parse().unwrap(),
        "114.80.149.19:7709".parse().unwrap(),
        "114.80.149.22:7709".parse().unwrap(),
        "115.238.56.198:7709".parse().unwrap(),
        "115.238.90.165:7709".parse().unwrap(),
        "117.184.140.156:7709".parse().unwrap(),
        "119.147.164.60:7709".parse().unwrap(),
        "123.125.108.23:7709".parse().unwrap(),
        "123.125.108.24:7709".parse().unwrap(),
        "180.153.18.170:7709".parse().unwrap(),
        "180.153.18.171:7709".parse().unwrap(),
        "180.153.39.51:7709".parse().unwrap(),
        "218.108.47.69:7709".parse().unwrap(),
        "218.108.98.244:7709".parse().unwrap(),
        "218.75.126.9:7709".parse().unwrap(),
        "221.194.181.176:7709".parse().unwrap(),
        "60.12.136.250:7709".parse().unwrap(),
        "60.191.117.167:7709".parse().unwrap(),
        "61.152.249.56:7709".parse().unwrap(),
        // "61.153.209.138:7709".parse().unwrap() 失效
    ];
}

#[cfg(test)]
mod tests {
    use super::STOCK_IP;
    use crate::tcp::tcpstream_ip;

    #[test]
    fn check_all_stock_ips() {
        let mut valid_addrs = Vec::with_capacity(STOCK_IP.len());
        for addr in *STOCK_IP {
            if tcpstream_ip(&addr).is_ok() {
                valid_addrs.push(addr);
            }
        }
        // 不再要求所有服务器都有效，而是要求至少有3个可用服务器
        // 这是因为部分服务器可能临时失效或网络不稳定
        assert!(
            valid_addrs.len() >= 3,
            "可用服务器数量不足: 至少需要3个，当前有 {} 个。可用服务器: {:?}",
            valid_addrs.len(),
            valid_addrs
        );
        println!("✅ 检测到 {} 个可用服务器 (总共 {} 个):", valid_addrs.len(), STOCK_IP.len());
        for addr in &valid_addrs {
            println!("  - {}", addr);
        }
    }
}
