use crate::tcp::{helper::price, Tdx};
use crate::bytes_helper::u16_from_le_bytes;

/// 获取股票成交明细。对应于 pytdx 中的 hq.get_transaction_data、GetTransactionDataCmd。
///
/// ## 注意
/// - 返回逐笔成交数据（tick级别）
/// - 每次请求最多返回指定数量的成交记录
/// - 支持分页获取（通过start参数）
/// - market: 0=深市, 1=沪市
///
/// ## 示例
/// ```ignore
/// use rustdx::tcp::{Tcp, Tdx};
/// use rustdx::tcp::stock::Transaction;
///
/// let mut tcp = Tcp::new()?;
/// let mut transaction = Transaction::new(0, "000001", 0, 20);
/// transaction.recv_parsed(&mut tcp)?;
/// for data in transaction.result().iter().take(10) {
///     println!("{} 价格:{:.2} 成交量:{}",
///         data.time, data.price, data.vol);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Transaction<'d> {
    pub send: Box<[u8]>,
    pub market: u16,
    pub code: &'d str,
    pub start: u16,
    pub count: u16,
    pub response: Vec<u8>,
    pub data: Vec<TransactionData>,
}

impl<'d> Transaction<'d> {
    /// 创建一个新的成交明细请求。
    ///
    /// ## 参数
    /// - `market`: 市场代码（0=深市, 1=沪市）
    /// - `code`: 6位股票代码
    /// - `start`: 起始位置，用于分页（0, 20, 40...）
    /// - `count`: 获取数量（建议每次20-50笔）
    pub fn new(market: u16, code: &'d str, start: u16, count: u16) -> Self {
        assert_eq!(code.len(), 6, "股票代码必须是6位");

        let mut send = [0u8; Self::LEN];
        // 复制包头（12字节）
        send[0..12].copy_from_slice(Self::SEND);

        // 设置market（字节12-13）
        send[12..14].copy_from_slice(&market.to_le_bytes());
        // 设置code（字节14-19）
        send[14..20].copy_from_slice(code.as_bytes());
        // 设置start（字节20-21）
        send[20..22].copy_from_slice(&start.to_le_bytes());
        // 设置count（字节22-23）
        send[22..24].copy_from_slice(&count.to_le_bytes());

        Self {
            send: send.into(),
            market,
            code,
            start,
            count,
            response: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<'a> Tdx for Transaction<'a> {
    type Item = [TransactionData];

    /// 获取成交明细的请求字节。
    ///
    /// ## 协议格式（基于pytdx源码分析）
    /// - 前12字节：固定包头
    /// - 字节12-13：market（市场代码）
    /// - 字节14-19：code（股票代码，6字节）
    /// - 字节20-21：start（起始位置）
    /// - 字节22-23：count（获取数量）
    const SEND: &'static [u8] = &[
        0x0c, 0x17, 0x08, 0x01, 0x01, 0x01, 0x0e, 0x00, 0x0e, 0x00, 0xc5,
        0x0f, // 固定包头（12字节）
    ];

    const TAG: &'static str = "成交明细";
    const LEN: usize = 12 + 2 + 6 + 2 + 2; // 固定长度：包头12 + market(2) + code(6) + start(2) + count(2)

    fn send(&mut self) -> &[u8] {
        &self.send
    }

    /// 解析响应的字节。
    ///
    /// ## 响应格式（基于pytdx源码分析）
    /// - 前2字节：成交笔数
    /// - 之后每笔成交：可变长度编码
    fn parse(&mut self, v: Vec<u8>) {
        let mut pos = 0;

        // 读取成交笔数
        let num_ticks = u16_from_le_bytes(&v, pos);
        pos += 2;

        self.data = Vec::with_capacity(num_ticks as usize);

        let mut last_price = 0i32;

        for _ in 0..num_ticks {
            // 解析时间（2字节：分钟数）
            let time_minutes = u16_from_le_bytes(&v, pos);
            pos += 2;
            let hour = time_minutes / 60;
            let minute = time_minutes % 60;

            // 解析成交明细（可变长度编码）
            let price_raw = price(&v, &mut pos);
            let vol = price(&v, &mut pos);
            let num = price(&v, &mut pos);
            let buyorsell = price(&v, &mut pos);
            let _reserved = price(&v, &mut pos);

            // 累加计算实际价格
            last_price += price_raw;
            let price = last_price as f64 / 100.0;

            self.data.push(TransactionData {
                time: format!("{:02}:{:02}", hour, minute),
                price,
                vol,
                num,
                buyorsell,
            });
        }

        self.response = v;
    }

    fn result(&self) -> &Self::Item {
        &self.data
    }
}

/// 成交明细数据点。
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct TransactionData {
    /// 时间（HH:MM格式）
    pub time: String,
    /// 价格（元）
    pub price: f64,
    /// 成交量（手）
    pub vol: i32,
    /// 成交编号
    pub num: i32,
    /// 买卖方向（0=买, 1=卖, 8=其他）
    pub buyorsell: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let transaction = Transaction::new(0, "000001", 0, 20);
        assert_eq!(transaction.market, 0);
        assert_eq!(transaction.code, "000001");
        assert_eq!(transaction.start, 0);
        assert_eq!(transaction.count, 20);
        assert_eq!(transaction.send.len(), 24);
    }

    #[test]
    fn test_transaction_new_with_start() {
        let transaction = Transaction::new(1, "600000", 20, 50);
        assert_eq!(transaction.market, 1);
        assert_eq!(transaction.code, "600000");
        assert_eq!(transaction.start, 20);
        assert_eq!(transaction.count, 50);
    }

    #[test]
    fn test_transaction_send_bytes() {
        let transaction = Transaction::new(0, "000001", 0, 20);
        // 验证包头
        assert_eq!(&transaction.send[0..12], &[0x0c, 0x17, 0x08, 0x01, 0x01, 0x01, 0x0e, 0x00, 0x0e, 0x00, 0xc5, 0x0f]);
        // 验证market
        assert_eq!(&transaction.send[12..14], &[0x00, 0x00]);
        // 验证code
        assert_eq!(&transaction.send[14..20], b"000001");
        // 验证start
        assert_eq!(&transaction.send[20..22], &[0x00, 0x00]);
        // 验证count = 20
        assert_eq!(&transaction.send[22..24], &[0x14, 0x00]);
    }

    #[test]
    #[should_panic(expected = "股票代码必须是6位")]
    fn test_transaction_invalid_code() {
        Transaction::new(0, "00001", 0, 20);
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
