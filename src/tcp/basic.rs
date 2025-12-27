use super::{u16_from_le_bytes, Result, Tdx};

/// 心跳包。用于保持 Tcp 连接。
pub type Heartbeat = SecurityCount;

/// 对应 pytdx 的 hq.security_count / GetSecurityCountCmd
/// （深沪市证券数量，包括指数、股票和大量债券）、心跳包。
#[derive(Debug)]
pub struct SecurityCount {
    send: Box<[u8]>,
    /// 0 代表深市；1 代表沪市。
    market: u16,
    /// 响应的结果：证券数量
    count: u16,
}

impl SecurityCount {
    /// market = 0 或 1，表示深市或沪市。
    pub fn new(market: u16) -> Self {
        let mut send = [0; Self::LEN];
        send.copy_from_slice(Self::SEND);
        if market != 0 {
            send[12..14].copy_from_slice(&market.to_le_bytes());
        }
        Self {
            send: send.into(),
            market,
            count: 0,
        }
    }

    pub fn market(&mut self, market: u16) {
        self.market = market;
        self.send[12..14].copy_from_slice(&market.to_le_bytes());
    }
}

impl Tdx for SecurityCount {
    type Item = u16;

    /// 深市证券数量的请求字节。
    const SEND: &'static [u8] = &[
        0x0c, 0x0c, 0x18, 0x6c, 0x00, 0x01, 0x08, 0x00, 0x08, 0x00, 0x4e, 0x04, 0x00, 0x00, 0x75,
        0xc7, 0x33, 0x01,
    ];
    const TAG: &'static str = "heartbeat";

    fn send(&mut self) -> &[u8] {
        &self.send
    }

    fn parse(&mut self, response: Vec<u8>) {
        self.count = u16_from_le_bytes(&response, 0);
    }

    fn result(&self) -> &Self::Item {
        &self.count
    }
}

/// 查询证券列表。对应于 pytdx 中的 GetSecurityList。
///
/// ## 注意：
/// - 获取的数据可能同一类别内是有序的，不同类别间是顺序未知 （比如 A
///   股股票之后不是创业板/科创板股票）；
/// - 每次返回 1000 条结果。
#[derive(Debug, Clone)]
pub struct SecurityList {
    pub send: Box<[u8]>,
    pub market: u16,
    pub start: u16,
    /// 响应信息中的列表长度。
    pub count: usize,
    pub response: Vec<u8>,
    pub data: Box<[SecurityListData]>,
}

impl Default for SecurityList {
    fn default() -> Self {
        Self {
            send: {
                let mut arr = [0; Self::LEN];
                arr.copy_from_slice(Self::SEND);
                arr.into()
            },
            market: 0,
            start: 1,
            count: 0,
            response: Vec::new(),
            data: [].into(),
        }
    }
}

impl SecurityList {
    /// 参数说明：
    /// - market = 0 或 1，表示深市或沪市；
    /// - start 在 [0, n] 的范围内，其中 n 是 [`SecurityCount`] 得到的结果。 目前 market = 0
    ///   时，有 13471 条； market = 1 时，有 18065 条。
    pub fn new(market: u16, start: u16) -> Self {
        Self {
            send: {
                let mut arr = [0; Self::LEN];
                arr.copy_from_slice(Self::SEND);
                arr[12..14].copy_from_slice(&market.to_le_bytes());
                arr[14..16].copy_from_slice(&start.to_le_bytes());
                arr.into()
            },
            market,
            start,
            count: 0,
            response: Vec::new(),
            data: [].into(),
        }
    }
}

impl Tdx for SecurityList {
    type Item = [SecurityListData];

    /// market = 0, start = 0 表示深市所有证券从第 0 个开始的 1000 条证券; 共 16 字节。
    const SEND: &'static [u8] = &[
        0x0c, 0x01, 0x18, 0x64, 0x01, 0x01, 0x06, 0x00, 0x06, 0x00, 0x50, 0x04, 0x00, 0x00, 0x00,
        0x00,
    ];
    const TAG: &'static str = "股票、指数列表";

    fn send(&mut self) -> &[u8] {
        &self.send
    }

    /// 前 2 字节表示列表的长度，剩余字节中，每 29 字节使用 [`SecurityListData::parse`] 解析。
    fn parse(&mut self, v: Vec<u8>) {
        self.count = u16_from_le_bytes(&v, 0) as usize;
        self.data = v[2..]
            .chunks_exact(29)
            .map(SecurityListData::parse)
            .collect();
        debug_assert_eq!(self.count, self.data.len());
        self.response = v;
    }

    fn result(&self) -> &Self::Item {
        &self.data
    }
}

#[test]
fn connection() -> Result<()> {
    // 跳过集成测试，如果设置了环境变量
    if std::env::var("RUSTDX_SKIP_INTEGRATION_TESTS").is_ok() {
        println!("⚠️  跳过集成测试 (RUSTDX_SKIP_INTEGRATION_TESTS 已设置)");
        return Ok(());
    }

    SecurityList::default().recv_parsed(&mut crate::tcp::Tcp::new()?)?;
    Ok(())
}

/// [`SecurityList`] 的解析结果。具体为指数、股票、债券等证券的代码、名称。
///
/// ## 注意
/// 有些响应的字节没有被解析，具体查看 [`SecurityListData::parse`] 的说明。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityListData {
    pub code: String,
    /// `\u0000` 字符表示空格
    pub name: String,
}

impl SecurityListData {
    /// 解析 [`SecurityList`] 的响应字节。传入长度为 29 字节序列。
    /// ```python
    /// (
    ///     code,            # UTF-8 编码
    ///     volunit,         # 100
    ///     name_bytes,      # GBK 编码
    ///     reversed_bytes1,
    ///     decimal_point,   # 2
    ///     pre_close_raw,   # 少许结果与实际数据有出入，故不解析
    ///     reversed_bytes2,
    /// ) = struct.unpack("<6sH8s4sBI4s", bytes) # python 表示方式
    /// ```
    pub fn parse(bytes: &[u8]) -> Self {
        let code = unsafe { std::str::from_utf8_unchecked(&bytes[0..6]) }.into();
        let (name, encoding_used, had_errors) = encoding_rs::GBK.decode(&bytes[8..16]);
        debug_assert_eq!(encoding_used, encoding_rs::GBK);
        debug_assert!(!had_errors);
        // let preclose = crate::tcp::helper::vol_amount(u32_from_le_bytes(b, 21) as i32);
        Self {
            code,
            name: name.into(),
        }
    }
}

pub const PACK1: &[u8] = &[
    0x0c, 0x02, 0x18, 0x93, 0x00, 0x01, 0x03, 0x00, 0x03, 0x00, 0x0d, 0x00, 0x01,
];
pub const PACK2: &[u8] = &[
    0x0c, 0x02, 0x18, 0x94, 0x00, 0x01, 0x03, 0x00, 0x03, 0x00, 0x0d, 0x00, 0x02,
];
pub const PACK3: &[u8] = &[
    0x0c, 0x03, 0x18, 0x99, 0x00, 0x01, 0x20, 0x00, 0x20, 0x00, 0xdb, 0x0f, 0xd5, 0xd0, 0xc9, 0xcc,
    0xd6, 0xa4, 0xa8, 0xaf, 0x00, 0x00, 0x00, 0x8f, 0xc2, 0x25, 0x40, 0x13, 0x00, 0x00, 0xd5, 0x00,
    0xc9, 0xcc, 0xbd, 0xf0, 0xd7, 0xea, 0x00, 0x00, 0x00, 0x02,
];
pub const RECV_SIZE: usize = 16;

pub fn send_packs(tcp: &mut super::Tcp, decompress: bool) -> Result<()> {
    use super::{send_recv, send_recv_decompress};
    if decompress {
        send_recv_decompress(tcp, PACK1, "PACK1")?;
        send_recv_decompress(tcp, PACK2, "PACK2")?;
        send_recv_decompress(tcp, PACK3, "PACK3")?;
    } else {
        send_recv(tcp, PACK1, "PACK1")?;
        send_recv(tcp, PACK2, "PACK2")?;
        send_recv(tcp, PACK3, "PACK3")?;
    }
    Ok(())
}
