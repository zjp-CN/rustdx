use crate::tcp::Tdx;

/// 对应 pytdx 的 hq.get_xdxr_info、GetXdXrInfo。获取单只股票的股本变迁信息。
#[derive(Debug, Clone)]
pub struct Xdxr<'d> {
    pub send:     Box<[u8]>,
    pub market:   u16,
    pub code:     &'d str,
    pub response: Vec<u8>,
    pub count:    Option<usize>,
    pub data:     Vec<XdxrData>,
}

impl<'d> Default for Xdxr<'d> {
    fn default() -> Self {
        Self { send:     {
                   let mut arr = [0; Self::LEN];
                   arr.copy_from_slice(Self::SEND);
                   arr.into()
               },
               market:   0,
               code:     "000001",
               response: Vec::new(),
               count:    None,
               data:     Vec::new(), }
    }
}

impl<'a> Tdx for Xdxr<'a> {
    type Item = [XdxrData];

    // market=0; code="000001"
    const SEND: &'static [u8] = &[0x0c, 0x1f, 0x18, 0x76, 0x00, 0x01, 0x0b, 0x00, 0x0b, 0x00,
                                  0x0f, 0x00, 0x01, 0x00, 0x00, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31];
    const TAG: &'static str = "除权除息";

    fn send(&mut self) -> &[u8] { &self.send }

    fn parse(&mut self, v: Vec<u8>) {
        if v.len() < 11 {
            return;
        }
        let count = crate::bytes_helper::u16_from_le_bytes(&v, 9) as usize;
        let old = self.count.replace(count);
        if old.is_some() {
            self.data.resize_with(count, XdxrData::default);
            self.data
                .iter_mut()
                .zip(v[11..].chunks_exact(29).map(XdxrData::parse))
                .map(|(d, x)| *d = x)
                .last();
        } else {
            self.data = Vec::with_capacity(count);
            v[11..].chunks_exact(29).map(XdxrData::parse).map(|x| self.data.push(x)).last();
        };
        self.response = v;
    }

    fn result(&self) -> &Self::Item { &self.data }
}

#[derive(Debug, Clone, Default)]
pub struct XdxrData {
    pub market:   u8,
    /// 6 位股票代码
    pub code:     String,
    /// 日期
    pub date:     u32,
    /// 信息类型
    ///
    /// |   | 类别                           |    | 类别                                    |
    /// | - | ------------------------------ | -- | --------------------------------------- |
    /// | 1 | 除权除息                       | 8  | 增发新股上市                            |
    /// | 2 | 送配股上市                     | 9  | 转配股上市                              |
    /// | 3 | 非流通股上市股                 | 10 | 可转债上市                              |
    /// | 4 | 未知股本变动                   | 11 | 扩缩股 songgu:比例                      |
    /// | 5 | 股本变化                       | 12 | 非流通股缩股 songgu: 比例               |
    /// | 6 | 增发新股 peigujia; songgu:未知 | 13 | 送认购权证 songgu: 份数; hongli: 行权价 |
    /// | 7 | 股份回购                       | 14 | 送认沽权证 songgu: 份数; hongli: 行权价 |
    pub category: u8,
    /// 分红（每 10 股派现金 x 元）| 前流通盘：
    pub fh_qltp:  f32,
    /// 配股价（每股配股价 x 元）| 前总股本
    pub pgj_qzgb: f32,
    /// 送转股（每 10 股送转股比例 x 股） | 后流通盘
    pub sg_hltp:  f32,
    /// 配股（每 10 股配股比例 x 股）| 后总股本：
    pub pg_hzgb:  f32,
}

impl XdxrData {
    /// 解析方式：
    ///
    /// | 位置 | 0    | 1-7  | 7 | 8-12             | 13   | 13-29    |
    /// | ---- | ---- | ---- | - | ---------------- | ---- | -------- |
    /// | 类型 | u8   | &str | / | [`DateTime`][DT] | u8   | /        |
    /// | 含义 | 市场 | 代码 | / | 日期             | 类别 | 四组数据 |
    ///
    /// [DT]: crate::tcp::helper::DateTime
    /// ## 注意
    /// `bytes` 为长度 29 的 slice
    pub fn parse(bytes: &[u8]) -> XdxrData {
        use crate::bytes_helper::{f32_from_le_bytes, u32_from_le_bytes, u8_from_le_bytes};
        fn f32_(b: &[u8], p: usize) -> f32 {
            let tmp = u32_from_le_bytes(b, p);
            if tmp == 0 {
                0.
            } else {
                crate::tcp::helper::vol_amount(tmp as i32) as f32
            }
        }
        let market = u8_from_le_bytes(bytes, 0);
        let code = unsafe { std::str::from_utf8_unchecked(&bytes[1..7]) }.into();
        let date = crate::tcp::helper::datetime(&bytes[8..12], 9).to_u32();
        let category = u8_from_le_bytes(bytes, 12);
        let (fh_qltp, pgj_qzgb, sg_hltp, pg_hzgb) = match category {
            1 | 11..=14 => (f32_from_le_bytes(bytes, 13),
                            f32_from_le_bytes(bytes, 17),
                            f32_from_le_bytes(bytes, 21),
                            f32_from_le_bytes(bytes, 25)),
            _ => (f32_(bytes, 13), f32_(bytes, 17), f32_(bytes, 21), f32_(bytes, 25)),
        };
        Self { market,
               code,
               date,
               category,
               fh_qltp,
               pgj_qzgb,
               sg_hltp,
               pg_hzgb }
    }
}

#[test]
fn connection() -> std::io::Result<()> { crate::tcp::tests::connection(Xdxr::default()) }

#[test]
fn xdxrdata_parse() {
    let target = XdxrData { market:   0,
                            code:     "000001".to_string(),
                            date:     19900301,
                            category: 1,
                            fh_qltp:  0.0,
                            pgj_qzgb: 3.56,
                            sg_hltp:  0.0,
                            pg_hzgb:  1.0, };
    let parsed = XdxrData::parse(&[0x00, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x00, 0x8d, 0xa7,
                                   0x2f, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x0a, 0xd7, 0x63,
                                   0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f]);
    compare!(parsed, target);
}
