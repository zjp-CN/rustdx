use crate::tcp::{helper::DateTime, Tdx};

// ['获取股票行情', '参数：市场代码， 股票代码， 如： 0,000001 或 1,  600300',
// get_security_quotes, '0,000001']),          (2, ['获取k线', '''category-> K线种类  0
// 5分钟K线 1 15分钟K线 2 30分钟K线 3 1小时K线 4 日K线  5 周K线   6 月K线
//   7 1分钟
//   8 1分钟K线 9 日K线
//   10 季K线
//   11 年K线
//   market -> 市场代码 0:深圳，1:上海
//   stockcode -> 证券代码;
//   start -> 指定的范围开始位置;
//   count -> 用户要请求的 K 线数目，最大值为 800
//
//   如： 9,0,000001,0,100''', get_security_bars, '9,0,000001,0,100']),
/// 查询股票日线。对应于 pytdx 中的 hq.get_security_bars、GetSecurityBarsCmd。
/// ## 注意
/// 只修改字段并不会更改待发送字节的内容。
/// 如果你需要修改查询条件，请使用相应的方法。
/// 比如修改股票代码调用 [`Kline::code`]，修改查询数量调用 [`Kline::count`]。
#[derive(Debug, Clone)]
pub struct Kline<'d> {
    pub send:     Box<[u8]>,
    pub market:   u16,
    pub code:     &'d str,
    pub category: u16,
    pub start:    u16,
    pub count:    u16,
    pub response: Vec<u8>,
    pub data:     Vec<KlineData<'d>>,
}

/// 为了对应 [`Kline::SEND`] 的含义，以下默认值值得注意：
/// 1. category 默认为 9 （日线）；
/// 2. code 默认为 `000001`；
/// 3. count 默认为 3；
/// 4. KlineData.dt.hour 默认小时数为 15。
#[rustfmt::skip]
impl<'d> Default for Kline<'d> {
    fn default() -> Self {
        Self { market: 0, code: "000001", category: 9, start: 0, count: 3,
               send:     { let mut v = [0; Self::LEN]; v.copy_from_slice(Self::SEND); v.into() },
               response: Vec::new(),
               data:     vec![KlineData::default(); 3], }
    }
}

impl<'d> Kline<'d> {
    /// 0 代表深市；1 代表沪市。
    ///
    /// ## panic
    /// 当 code 的字节长度不是 6 时，程序会 panic。
    #[rustfmt::skip]
    pub fn new(market: u16, code: &'d str, category: u16, start: u16, count: u16) -> Self {
        Self { market, code, category, start, count,
               send: {
                   let mut arr = [0; Self::LEN];
                   arr.copy_from_slice(Self::SEND);
                   arr[12..14].copy_from_slice(&market.to_le_bytes());
                   arr[14..20].copy_from_slice(code.as_bytes());
                   arr[20..22].copy_from_slice(&category.to_le_bytes());
                   arr[24..26].copy_from_slice(&start.to_le_bytes());
                   arr[26..28].copy_from_slice(&count.to_le_bytes());
                   arr.into()
               },
               response: Vec::new(),
               data: vec![KlineData::default(); count as usize] }
    }

    /// 修改市场。
    pub fn market(&mut self, market: u16) -> &mut Self {
        self.market = market;
        self.send[12..14].copy_from_slice(&market.to_le_bytes());
        self
    }

    /// 修改股票。当代码不正确时，不能正常得到响应。
    ///
    /// ## panic
    /// 当 code 的字节长度不是 6 时，程序会 panic。
    pub fn code(&mut self, code: &'d str) -> &mut Self {
        self.code = code;
        self.send[14..20].copy_from_slice(code.as_bytes());
        self
    }

    /// 修改 K 线类型。
    pub fn category(&mut self, category: u16) -> &mut Self {
        self.category = category;
        self.send[20..22].copy_from_slice(&category.to_le_bytes());
        self
    }

    /// 修改起始位置。
    pub fn start(&mut self, start: u16) -> &mut Self {
        self.start = start;
        self.send[24..26].copy_from_slice(&start.to_le_bytes());
        self
    }

    /// 修改查询数量。
    ///
    /// 注意：此方法调用了 [`Vec::resize_with`]
    /// 1. 如果此次修改的查询数量大于修改前的查询数量，
    ///    则 data 字段除了保持已有的数据，还会会增加默认值。
    /// 2. 如果此次修改的查询数量小于或等于修改前的查询数量，
    ///    则 data 字段截断已有的数据。
    /// 3. 无论以上哪种情况，目的都是保证查询数量与 data 字段的 Vec 长度一致。
    pub fn count(&mut self, count: u16) -> &mut Self {
        self.data.resize_with(count as usize, Default::default);
        self.count = count;
        self.send[26..28].copy_from_slice(&count.to_le_bytes());
        self
    }
}

impl<'a> Tdx for Kline<'a> {
    type Item = [KlineData<'a>];

    /// #sz000001# 最近三天日线的请求字节。长度为 38。
    /// ```python
    /// struct.pack("<HIHHHH6sHHHHIIH", bytes) # python 中的解读方式
    /// ```
    const SEND: &'static [u8] = &[0x0c, 0x01, 0x08, 0x64, 0x01, 0x01, 0x1c, 0x00, 0x1c, 0x00,
                                  0x2d, 0x05, 0x00, 0x00, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31,
                                  0x09, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
                                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    const TAG: &'static str = "日线";

    fn send(&mut self) -> &[u8] { &self.send }

    #[rustfmt::skip]
    fn parse(&mut self, v: Vec<u8>) {
        use crate::{
            tcp::helper::{datetime, price, vol_amount},
            bytes_helper::{u16_from_le_bytes, u32_from_le_bytes}
        };

        let (count, mut pos, mut base) = (u16_from_le_bytes(&v, 0), 2, 0);
        debug_assert_eq!(count, self.count);
        for item in self.data.iter_mut() {
            let dt = datetime(&v[pos..pos + 4], self.category);
            pos += 4;
            let open = price(&v, &mut pos);
            let close = price(&v, &mut pos);

            *item = KlineData { dt, code: self.code,
                                open:   { base += open; base as f64 / 1000. },
                                close:  real_price(close, base),
                                high:   real_price(price(&v, &mut pos), base),
                                low:    real_price(price(&v, &mut pos), base),
                                vol:    { pos += 4; vol_amount(u32_from_le_bytes(&v, pos - 4) as i32) },
                                amount: { pos += 4; vol_amount(u32_from_le_bytes(&v, pos - 4) as i32) }};

            base += close;
        }
        self.response = v;
    }

    fn result(&self) -> &Self::Item { &self.data }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct KlineData<'d> {
    pub dt:     DateTime,
    pub code:   &'d str,
    pub open:   f64,
    pub close:  f64,
    pub high:   f64,
    pub low:    f64,
    /// 成交量，单位：股
    pub vol:    f64,
    /// 成交额，单位：元
    pub amount: f64,
}

// impl<'d> KlineData<'d> {
//     pub fn parse(v: &'d [u8], mut pos: usize, mut base: i32, code: &'d str, category: u16)
//                  -> (usize, i32, KlineData<'d>) {
//         use crate::{
//             bytes_helper::u32_from_le_bytes,
//             tcp::helper::{datetime, price, vol_amount},
//         };
//         let dt = datetime(&v[pos..pos + 4], category);
//         pos += 4;
//         let open = price(&v, &mut pos);
//         let close = price(&v, &mut pos);
//
//         #[rustfmt::skip]
//         let kline = Self { dt, code,
//                            open: { base += open; base as f64 / 1000.  },
//                            close: real_price(close, base),
//                            high: real_price(price(&v, &mut pos), base),
//                            low: real_price(price(&v, &mut pos), base),
//                            vol: { pos += 4; vol_amount(u32_from_le_bytes(&v, pos - 4) as
// i32) },                            amount: { pos += 4; vol_amount(u32_from_le_bytes(&v, pos
// - 4) as i32) } };
//
//         base += close;
//         (pos, base, kline)
//     }
// }

#[inline]
fn real_price(p: i32, base: i32) -> f64 { (p + base) as f64 / 1000. }

#[test]
fn day_new_modify() {
    let day = Kline::new(0, "000001", 9, 0, 3);
    let mut day2 = Kline::new(1, "000000", 0, 1, 1);
    day2.market(0).code("000001").category(9).start(0).count(3);
    compare!(Kline::default(), day, day2);
}

#[test]
fn connection() -> std::io::Result<()> { crate::tcp::tests::connection(Kline::default()) }

#[test]
fn parse() {
    let mut day = Kline::default();
    let arr = vec![0x03, 0x00, 0xeb, 0x64, 0x34, 0x01, 0xb4, 0x9a, 0x02, 0xe4, 0x06, 0x9c, 0x03,
                   0xc2, 0x07, 0xe8, 0x6f, 0xa8, 0x49, 0x59, 0xf7, 0x12, 0x4f, 0xec, 0x64, 0x34,
                   0x01, 0xd0, 0x01, 0xfa, 0x03, 0x90, 0x01, 0xc4, 0x04, 0x00, 0x81, 0x9a, 0x49,
                   0xb7, 0xb1, 0x03, 0x4f, 0xef, 0x64, 0x34, 0x01, 0xcc, 0x02, 0xa8, 0x05, 0x96,
                   0x07, 0xd6, 0x02, 0xd8, 0x3d, 0x8b, 0x49, 0x4b, 0xf0, 0xeb, 0x4e];
    let res = [KlineData { dt:     DateTime { year:   2021,
                                              month:  9,
                                              day:    23,
                                              hour:   15,
                                              minute: 0, },
                           code:   "000001",
                           open:   18.1,
                           close:  17.68,
                           high:   18.32,
                           low:    17.65,
                           vol:    1379837.0,
                           amount: 2465683712.0, },
               KlineData { dt:     DateTime { year:   2021,
                                              month:  9,
                                              day:    24,
                                              hour:   15,
                                              minute: 0, },
                           code:   "000001",
                           open:   17.6,
                           close:  17.35,
                           high:   17.68,
                           low:    17.34,
                           vol:    1265696.0,
                           amount: 2209462016.0, },
               KlineData { dt:     DateTime { year:   2021,
                                              month:  9,
                                              day:    27,
                                              hour:   15,
                                              minute: 0, },
                           code:   "000001",
                           open:   17.21,
                           close:  17.57,
                           high:   17.68,
                           low:    17.06,
                           vol:    1140667.0,
                           amount: 1979196800.0, }];
    day.parse(arr);
    compare!(res, day.data.as_slice());
}
