use crate::tcp::{helper::price, Tdx};
use crate::bytes_helper::u16_from_le_bytes;

/// 获取股票分时数据。对应于 pytdx 中的 hq.get_minute_time_data、GetMinuteTimeDataCmd。
///
/// ## 注意
/// - 返回当天的分时成交数据（每分钟一个数据点）
/// - 通常返回240个数据点（4小时交易时间）
/// - market: 0=深市, 1=沪市
///
/// ## 示例
/// ```ignore
/// use rustdx::tcp::{Tcp, Tdx};
/// use rustdx::tcp::stock::MinuteTime;
///
/// let mut tcp = Tcp::new()?;
/// let mut minute = MinuteTime::new(0, "000001");
/// minute.recv_parsed(&mut tcp)?;
/// for data in minute.result().iter().take(10) {
///     println!("价格: {:.2}, 成交量: {}", data.price, data.vol);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MinuteTime<'d> {
    pub send: Box<[u8]>,
    pub market: u16,
    pub code: &'d str,
    pub response: Vec<u8>,
    pub data: Vec<MinuteTimeData>,
}

impl<'d> MinuteTime<'d> {
    /// 创建一个新的分时数据请求。
    ///
    /// ## 参数
    /// - `market`: 市场代码（0=深市, 1=沪市）
    /// - `code`: 6位股票代码
    pub fn new(market: u16, code: &'d str) -> Self {
        assert_eq!(code.len(), 6, "股票代码必须是6位");

        let mut send = [0u8; Self::LEN];
        // 复制包头（12字节）
        send[0..12].copy_from_slice(Self::SEND);

        // 设置market（字节12-13）
        send[12..14].copy_from_slice(&market.to_le_bytes());
        // 设置code（字节14-19）
        send[14..20].copy_from_slice(code.as_bytes());
        // 字节20-23：设置为0

        Self {
            send: send.into(),
            market,
            code,
            response: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<'a> Tdx for MinuteTime<'a> {
    type Item = [MinuteTimeData];

    /// 获取分时数据的请求字节。
    ///
    /// ## 协议格式（基于pytdx源码分析）
    /// - 前12字节：固定包头
    /// - 字节12-13：market（市场代码）
    /// - 字节14-19：code（股票代码，6字节）
    /// - 字节20-23：0
    const SEND: &'static [u8] = &[
        0x0c, 0x1b, 0x08, 0x00, 0x01, 0x01, 0x0e, 0x00, 0x0e, 0x00, 0x1d,
        0x05, // 固定包头（12字节）
    ];

    const TAG: &'static str = "分时数据";
    const LEN: usize = 12 + 2 + 6 + 4; // 固定长度：包头12字节 + market(2) + code(6) + 0(4)

    fn send(&mut self) -> &[u8] {
        &self.send
    }

    /// 解析响应的字节。
    ///
    /// ## 响应格式（基于pytdx源码分析）
    /// - 前2字节：数据点数量
    /// - 字节2-3：跳过
    /// - 之后每个数据点：可变长度编码
    fn parse(&mut self, v: Vec<u8>) {
        let mut pos = 0;

        // 读取数据点数量
        let num_points = u16_from_le_bytes(&v, pos);
        pos += 4; // 跳过前4字节（数量 + 2字节跳过）

        self.data = Vec::with_capacity(num_points as usize);

        let mut last_price = 0i32;

        for _ in 0..num_points {
            // 解析分时数据（可变长度编码）
            let price_raw = price(&v, &mut pos);
            let _reversed1 = price(&v, &mut pos);
            let vol = price(&v, &mut pos);

            // 累加计算实际价格
            last_price += price_raw;
            let price = last_price as f64 / 100.0;

            self.data.push(MinuteTimeData { price, vol });
        }

        self.response = v;
    }

    fn result(&self) -> &Self::Item {
        &self.data
    }
}

/// 分时数据点。
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct MinuteTimeData {
    /// 价格（元）
    pub price: f64,
    /// 成交量（手）
    pub vol: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minute_time_new() {
        let minute = MinuteTime::new(0, "000001");
        assert_eq!(minute.market, 0);
        assert_eq!(minute.code, "000001");
        assert_eq!(minute.send.len(), 24);
    }

    #[test]
    fn test_minute_time_new_shanghai() {
        let minute = MinuteTime::new(1, "600000");
        assert_eq!(minute.market, 1);
        assert_eq!(minute.code, "600000");
    }

    #[test]
    fn test_minute_time_send_bytes() {
        let minute = MinuteTime::new(0, "000001");
        // 验证包头
        assert_eq!(&minute.send[0..12], &[0x0c, 0x1b, 0x08, 0x00, 0x01, 0x01, 0x0e, 0x00, 0x0e, 0x00, 0x1d, 0x05]);
        // 验证market
        assert_eq!(&minute.send[12..14], &[0x00, 0x00]);
        // 验证code
        assert_eq!(&minute.send[14..20], b"000001");
        // 验证最后的0
        assert_eq!(&minute.send[20..24], &[0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    #[should_panic(expected = "股票代码必须是6位")]
    fn test_minute_time_invalid_code() {
        MinuteTime::new(0, "00001");
    }

    #[test]
    fn test_connection() {
        // 跳过集成测试（需要实际网络连接）
        if std::env::var("RUSTDX_SKIP_INTEGRATION_TESTS").is_ok() {
            println!("⚠️  跳过集成测试 (RUSTDX_SKIP_INTEGRATION_TESTS 已设置)");
            return;
        }
        println!("⚠️  集成测试需要手动验证（需要实际TCP连接）");
    }
}
