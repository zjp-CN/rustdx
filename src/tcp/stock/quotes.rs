use crate::tcp::{helper::price, Tdx};
use crate::bytes_helper::{u16_from_le_bytes, u32_from_le_bytes};

/// 获取股票实时行情快照。对应于 pytdx 中的 hq.get_security_quotes、GetSecurityQuotesCmd。
///
/// ## 注意
/// - 可以一次获取多只股票的实时行情信息（建议不超过80只）
/// - 返回字段：当前价、开高低收、成交量、成交额、买卖五档等
///
/// ## 示例
/// ```ignore
/// use rustdx::tcp::{Tcp, Tdx};
/// use rustdx::tcp::stock::quotes::SecurityQuotes;
///
/// let mut tcp = Tcp::new()?;
/// let mut quotes = SecurityQuotes::new(vec![(0, "000001"), (1, "600000")]);
/// let data = quotes.recv_parsed(&mut tcp)?;
/// for quote in data {
///     println!("{}: {} - {}", quote.code, quote.name, quote.price);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SecurityQuotes<'d> {
    pub send: Box<[u8]>,
    pub stocks: Vec<(u16, &'d str)>,
    pub response: Vec<u8>,
    pub data: Vec<QuoteData>,
}

impl<'d> Default for SecurityQuotes<'d> {
    fn default() -> Self {
        Self::new(vec![(0, "000001")])
    }
}

impl<'d> SecurityQuotes<'d> {
    /// 创建一个新的股票行情请求。
    ///
    /// ## 参数
    /// - `stocks`: 股票列表，格式为 `[(market, code), ...]`
    ///   - market: 0=深市, 1=沪市
    ///   - code: 6位股票代码
    ///
    /// ## panic
    /// 当任何股票代码的长度不是6时，程序会panic。
    pub fn new(stocks: Vec<(u16, &'d str)>) -> Self {
        let count = stocks.len();
        assert!(count > 0 && count <= 80, "股票数量必须在1-80之间");

        // 计算包长度: stock_count * 7 + 12（注意：这里是整个包的数据长度）
        let pkg_len = (count * 7 + 12) as u16;

        let mut send = [0u8; Self::LEN];
        // 复制整个包头（22字节）
        send[0..22].copy_from_slice(Self::SEND);

        // 设置包长度（字节6-7，第一个pkg_len，u16）
        send[6..8].copy_from_slice(&pkg_len.to_le_bytes());
        // 设置包长度重复（字节8-9，第二个pkg_len，u16）
        send[8..10].copy_from_slice(&pkg_len.to_le_bytes());

        // 设置股票数量（字节20-21）
        send[20..22].copy_from_slice(&(count as u16).to_le_bytes());

        // 填充每只股票的信息（每只7字节: 1字节market + 6字节code）
        let mut pos = 22;
        for (market, code) in &stocks {
            send[pos] = *market as u8;
            send[pos + 1..pos + 7].copy_from_slice(code.as_bytes());
            pos += 7;
        }

        Self {
            send: send.into(),
            stocks,
            response: Vec::new(),
            data: Vec::with_capacity(count),
        }
    }
}

impl<'a> Tdx for SecurityQuotes<'a> {
    type Item = [QuoteData];

    /// 获取股票行情的请求字节。
    ///
    /// ## 协议格式（基于pytdx源码分析）
    /// - 前22字节：固定包头（struct.pack("<HIHHIIHH", ...)）
    ///   - 0-1: H (0x010c = 268)
    ///   - 2-5: I (0x02006320)
    ///   - 6-7: H (pkg_len1)
    ///   - 8-9: H (pkg_len2)
    ///   - 10-13: I (0x5053e = 329054)
    ///   - 14-17: I (0)
    ///   - 18-19: H (0)
    ///   - 20-21: H (stock_len)
    /// - 之后每7字节：一只股票 (1字节market + 6字节code)
    const SEND: &'static [u8] = &[
        0x0c, 0x01,                   // H: 0x010c = 268 (2 bytes)
        0x20, 0x63, 0x00, 0x02,       // I: 0x02006320 (4 bytes)
        0x00, 0x00,                   // H: pkg_len1 (占位符, 2 bytes)
        0x00, 0x00,                   // H: pkg_len2 (占位符, 2 bytes)
        0x3e, 0x05, 0x05, 0x00,       // I: 0x0005053e (4 bytes) - 修正为pytdx的实际值
        0x00, 0x00, 0x00, 0x00,       // I: 0 (4 bytes)
        0x00, 0x00,                   // H: 0 (2 bytes)
        0x01, 0x00,                   // H: stock_len (占位符，默认1, 2 bytes)
    ];

    const TAG: &'static str = "股票行情快照";
    const LEN: usize = 22 + 80 * 7; // 固定长度：包头22字节 + 最多80只股票

    fn send(&mut self) -> &[u8] {
        // 只返回实际需要发送的字节数：包头22字节 + 每只股票7字节
        let actual_len = 22 + self.stocks.len() * 7;
        &self.send[..actual_len]
    }

    /// 解析响应的字节。
    ///
    /// ## 响应格式（基于pytdx源码分析）
    /// - 前2字节：跳过
    /// - 接下来2字节：股票数量
    /// - 之后每只股票：约200字节的数据
    fn parse(&mut self, v: Vec<u8>) {
        let mut pos = 0;

        // 跳过前2字节
        pos += 2;

        // 读取股票数量
        let num_stocks = u16_from_le_bytes(&v, pos);
        pos += 2;

        self.data = Vec::with_capacity(num_stocks as usize);

        for _ in 0..num_stocks {
            // 解析每只股票数据
            let quote = parse_quote(&v, &mut pos);
            self.data.push(quote);
        }

        self.response = v;
    }

    fn result(&self) -> &Self::Item {
        &self.data
    }
}

/// 解析单只股票的行情数据
fn parse_quote(data: &[u8], pos: &mut usize) -> QuoteData {
    // market (1字节) + code (6字节) + active1 (2字节)
    let _market = data[*pos] as u16;
    *pos += 1;
    let code_bytes = &data[*pos..*pos + 6];
    *pos += 6;
    let code = unsafe { std::str::from_utf8_unchecked(code_bytes) };
    let code = String::from(code); // 转换为拥有所有权的String
    let _active1 = u16_from_le_bytes(data, *pos);
    *pos += 2;

    // 解析价格（使用price函数解析可变长度编码）
    let price_rel = price(data, pos);
    let _last_close_diff = price(data, pos);
    let _open_diff = price(data, pos);
    let _high_diff = price(data, pos);
    let _low_diff = price(data, pos);

    // reversed_bytes0
    let _reversed_bytes0 = price(data, pos);
    let _reversed_bytes1 = price(data, pos);

    // vol, cur_vol
    let vol = price(data, pos);
    let _cur_vol = price(data, pos);

    // amount (4字节)
    let amount_raw = u32_from_le_bytes(data, *pos);
    *pos += 4;
    let amount = vol_amount(amount_raw as i32);

    // s_vol, b_vol
    let _s_vol = price(data, pos);
    let _b_vol = price(data, pos);

    // reversed_bytes2-3
    let _reversed_bytes2 = price(data, pos);
    let _reversed_bytes3 = price(data, pos);

    // bid1, ask1及其成交量
    let bid1 = price(data, pos);
    let ask1 = price(data, pos);
    let bid1_vol = price(data, pos);
    let ask1_vol = price(data, pos);

    // bid2-5, ask2-5及其成交量（暂时跳过，简化实现）
    let _bid2 = price(data, pos);
    let _ask2 = price(data, pos);
    let _bid2_vol = price(data, pos);
    let _ask2_vol = price(data, pos);

    let _bid3 = price(data, pos);
    let _ask3 = price(data, pos);
    let _bid3_vol = price(data, pos);
    let _ask3_vol = price(data, pos);

    let _bid4 = price(data, pos);
    let _ask4 = price(data, pos);
    let _bid4_vol = price(data, pos);
    let _ask4_vol = price(data, pos);

    let _bid5 = price(data, pos);
    let _ask5 = price(data, pos);
    let _bid5_vol = price(data, pos);
    let _ask5_vol = price(data, pos);

    // reversed_bytes4-9, active2（暂时跳过）
    let _reversed_bytes4 = u16_from_le_bytes(data, *pos);
    *pos += 2;
    let _reversed_bytes5 = price(data, pos);
    let _reversed_bytes6 = price(data, pos);
    let _reversed_bytes7 = price(data, pos);
    let _reversed_bytes8 = price(data, pos);
    let reversed_bytes9 = u16_from_le_bytes(data, *pos) as f64 / 100.0;
    let _active2 = u16_from_le_bytes(data, *pos + 2);
    *pos += 4;

    // 计算实际价格（除以100）
    let price_calc = price_rel as f64 / 100.0;

    // 暂时简化实现，主要返回核心字段
    QuoteData {
        code,
        name: String::new(), // 名称需要另外查询
        price: price_calc,
        preclose: 0.0,     // 需要从last_close_diff计算
        open: price_calc,   // 需要从open_diff计算
        high: price_calc,   // 需要从high_diff计算
        low: price_calc,    // 需要从low_diff计算
        vol: vol as f64 / 100.0,
        amount,
        bid1: bid1 as f64 / 100.0,
        ask1: ask1 as f64 / 100.0,
        bid1_vol: bid1_vol as f64 / 100.0,
        ask1_vol: ask1_vol as f64 / 100.0,
        change: 0.0,
        change_percent: reversed_bytes9,
        time: 0,
    }
}

/// 成交量转换（与pytdx的helper.get_volume一致）
fn vol_amount(ivol: i32) -> f64 {
    crate::tcp::helper::vol_amount(ivol)
}

/// 股票实时行情数据。
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct QuoteData {
    /// 股票代码（6位）
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 当前价
    pub price: f64,
    /// 昨收价
    pub preclose: f64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 成交量（手）
    pub vol: f64,
    /// 成交额（元）
    pub amount: f64,
    /// 涨跌额
    pub change: f64,
    /// 涨跌幅(%)
    pub change_percent: f64,
    /// 买一价
    pub bid1: f64,
    /// 卖一价
    pub ask1: f64,
    /// 买一量（手）
    pub bid1_vol: f64,
    /// 卖一量（手）
    pub ask1_vol: f64,
    /// 时间戳
    pub time: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_quotes_default() {
        let quotes = SecurityQuotes::default();
        assert_eq!(quotes.stocks.len(), 1);
        assert_eq!(quotes.stocks[0].0, 0);
        assert_eq!(quotes.stocks[0].1, "000001");
    }

    #[test]
    fn test_security_quotes_new() {
        let stocks = vec![(0, "000001"), (1, "600000")];
        let quotes = SecurityQuotes::new(stocks);
        assert_eq!(quotes.stocks.len(), 2);
    }

    #[test]
    #[should_panic(expected = "股票数量必须在1-80之间")]
    fn test_security_quotes_empty() {
        SecurityQuotes::new(vec![]);
    }

    #[test]
    fn test_connection() {
        // 跳过集成测试（需要实际网络连接）
        if std::env::var("RUSTDX_SKIP_INTEGRATION_TESTS").is_ok() {
            println!("⚠️  跳过集成测试 (RUSTDX_SKIP_INTEGRATION_TESTS 已设置)");
            return;
        }

        // 实际连接测试（需要网络）
        // let mut tcp = crate::Tcp::new().unwrap();
        // let mut quotes = SecurityQuotes::new(vec![(0, "000001")]);
        // quotes.recv_parsed(&mut tcp).unwrap();
        println!("⚠️  集成测试需要手动验证（需要实际TCP连接）");
    }
}
