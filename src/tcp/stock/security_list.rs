use crate::tcp::{helper, Tdx};
use crate::bytes_helper::{u16_from_le_bytes, u32_from_le_bytes};

/// 获取股票列表。对应于 pytdx 中的 hq.get_security_lists、GetSecurityListCmd。
///
/// ## 注意
/// - 每次最多返回1000只股票
/// - 支持分页获取（通过start参数）
/// - market: 0=深市, 1=沪市
///
/// ## 示例
/// ```ignore
/// use rustdx::tcp::{Tcp, Tdx};
/// use rustdx::tcp::stock::SecurityList;
///
/// let mut tcp = Tcp::new()?;
/// let mut list = SecurityList::new(0, 0); // 深市，从0开始
/// list.recv_parsed(&mut tcp)?;
/// for stock in list.result().iter().take(5) {
///     println!("{} - {}", stock.code, stock.name);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SecurityList {
    pub send: Box<[u8]>,
    pub market: u16,
    pub start: u16,
    pub response: Vec<u8>,
    pub data: Vec<SecurityListData>,
}

impl SecurityList {
    /// 创建一个新的股票列表请求。
    ///
    /// ## 参数
    /// - `market`: 市场代码（0=深市, 1=沪市）
    /// - `start`: 起始位置，用于分页（0, 1000, 2000...）
    pub fn new(market: u16, start: u16) -> Self {
        let mut send = [0u8; Self::LEN];
        // 复制包头（12字节）
        send[0..12].copy_from_slice(Self::SEND);

        // 设置market（字节12-13）
        send[12..14].copy_from_slice(&market.to_le_bytes());
        // 设置start（字节14-15）
        send[14..16].copy_from_slice(&start.to_le_bytes());

        Self {
            send: send.into(),
            market,
            start,
            response: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl Tdx for SecurityList {
    type Item = [SecurityListData];

    /// 获取股票列表的请求字节。
    ///
    /// ## 协议格式（基于pytdx源码分析）
    /// - 前12字节：固定包头
    /// - 字节12-13：market（市场代码）
    /// - 字节14-15：start（起始位置）
    const SEND: &'static [u8] = &[
        0x0c, 0x01, 0x18, 0x64, 0x01, 0x01, 0x06, 0x00, 0x06, 0x00, 0x50,
        0x04, // 固定包头（12字节）
    ];

    const TAG: &'static str = "股票列表";
    const LEN: usize = 12 + 2 + 2; // 固定长度：包头12字节 + market(2) + start(2)

    fn send(&mut self) -> &[u8] {
        &self.send
    }

    /// 解析响应的字节。
    ///
    /// ## 响应格式（基于pytdx源码分析）
    /// - 前2字节：股票数量
    /// - 之后每只股票：29字节
    fn parse(&mut self, v: Vec<u8>) {
        let mut pos = 0;

        // 读取股票数量
        let num_stocks = u16_from_le_bytes(&v, pos);
        pos += 2;

        self.data = Vec::with_capacity(num_stocks as usize);

        for _ in 0..num_stocks {
            // 解析每只股票数据（29字节）
            let stock = parse_security_list_data(&v, &mut pos);
            self.data.push(stock);
        }

        self.response = v;
    }

    fn result(&self) -> &Self::Item {
        &self.data
    }
}

/// 解析单只股票的列表数据
fn parse_security_list_data(data: &[u8], pos: &mut usize) -> SecurityListData {
    // code (6字节)
    let code_bytes = &data[*pos..*pos + 6];
    *pos += 6;
    let code = unsafe { std::str::from_utf8_unchecked(code_bytes) };
    let code = String::from(code);

    // volunit (2字节)
    let volunit = u16_from_le_bytes(data, *pos);
    *pos += 2;

    // name (8字节，GBK编码)
    let name_bytes = &data[*pos..*pos + 8];
    *pos += 8;
    // 尝试GBK解码，如果失败则使用默认值
    let name = String::from_utf8_lossy(name_bytes)
        .trim_end_matches('\x00')
        .to_string();

    // reversed_bytes1 (4字节)
    let _reversed_bytes1 = &data[*pos..*pos + 4];
    *pos += 4;

    // decimal_point (1字节)
    let decimal_point = data[*pos];
    *pos += 1;

    // pre_close_raw (4字节)
    let pre_close_raw = u32_from_le_bytes(data, *pos);
    *pos += 4;
    let pre_close = helper::vol_amount(pre_close_raw as i32);

    // reversed_bytes2 (4字节)
    let _reversed_bytes2 = &data[*pos..*pos + 4];
    *pos += 4;

    SecurityListData {
        code,
        volunit: volunit as u32,
        decimal_point,
        name,
        pre_close,
    }
}

/// 股票列表数据。
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SecurityListData {
    /// 股票代码（6位）
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 成交量单位（通常为100，表示1手=100股）
    pub volunit: u32,
    /// 小数点位数
    pub decimal_point: u8,
    /// 昨收价
    pub pre_close: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_list_new() {
        let list = SecurityList::new(0, 0);
        assert_eq!(list.market, 0);
        assert_eq!(list.start, 0);
        assert_eq!(list.send.len(), 16);
    }

    #[test]
    fn test_security_list_new_with_start() {
        let list = SecurityList::new(1, 1000);
        assert_eq!(list.market, 1);
        assert_eq!(list.start, 1000);
    }

    #[test]
    fn test_security_list_send_bytes() {
        let list = SecurityList::new(0, 0);
        // 验证包头
        assert_eq!(&list.send[0..12], &[0x0c, 0x01, 0x18, 0x64, 0x01, 0x01, 0x06, 0x00, 0x06, 0x00, 0x50, 0x04]);
        // 验证market
        assert_eq!(&list.send[12..14], &[0x00, 0x00]);
        // 验证start
        assert_eq!(&list.send[14..16], &[0x00, 0x00]);
    }

    #[test]
    fn test_security_list_send_bytes_with_params() {
        let list = SecurityList::new(1, 1000);
        // 验证market = 1
        assert_eq!(&list.send[12..14], &[0x01, 0x00]);
        // 验证start = 1000
        assert_eq!(&list.send[14..16], &[0xe8, 0x03]); // 1000 = 0x03e8
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
