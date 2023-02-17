//! 对应于 pytdx/helper.py 文件，用于辅助解析响应的字节数据。

use crate::bytes_helper::{u16_from_le_bytes, u32_from_le_bytes};

/// 解析日期时间的原始结果。如果需要其他形式的日期时间，可自行转化。
///
/// 由函数 [`datetime`] 解析响应字节得到此结构体。
///
/// 注意：默认 15 时（即 `DateTime::default().hour == 15`）。
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DateTime {
    pub year:   u16,
    pub month:  u16,
    pub day:    u16,
    pub hour:   u16,
    pub minute: u16,
}

/// hour 默认为 15 。
#[rustfmt::skip]
impl Default for DateTime {
    fn default() -> Self { Self { year: 0, month: 0, day: 0, hour: 15, minute: 0, } }
}

impl DateTime {
    pub fn into_string(self, category: u16) -> String {
        match category {
            0..=3 | 7 | 8 => format!("{:04}-{:02}-{:02} {:02}:{:02}",
                                     self.year, self.month, self.day, self.hour, self.minute),
            _ => format!("{:04}-{:02}-{:02}", self.year, self.month, self.day),
        }
    }

    /// 转化成日期 u32 类型
    pub fn to_u32(self) -> u32 {
        self.day as u32 + self.month as u32 * 100 + self.year as u32 * 10000
    }
}

/// 解析日期和时间（小时分钟）。
///
/// 对于不需要时间的 category （比如 category = 9 的日线），以默认的 15:00 作为时间。
///  
/// 注意：参数 arr 为长度为 4 的 slice。
pub fn datetime(arr: &[u8], category: u16) -> DateTime {
    let mut datetime = DateTime::default();

    if category < 4 || category == 7 || category == 8 {
        let day = u16_from_le_bytes(arr, 0);
        let minutes = u16_from_le_bytes(arr, 2);
        datetime.year = (day >> 11) + 2004;
        datetime.month = day % 2048 / 100;
        datetime.day = day % 2048 % 100;
        datetime.hour = minutes / 60;
        datetime.minute = minutes % 60;
    } else {
        let day = u32_from_le_bytes(arr, 0);
        datetime.year = (day / 10000) as u16;
        datetime.month = (day % 10000 / 100) as u16;
        datetime.day = (day % 100) as u16;
    }

    datetime
}

/// 解析价格 (open / close / high / low)
///
/// 注意：
/// 1. 第二次之后计算的价格为浮动价格，基于第一次解析的实际价格而浮动；
/// 2. 返回的 pos 是不定长的。
pub fn price(arr: &[u8], pos: &mut usize) -> i32 {
    let mut shl = 6;
    let mut bit = arr[*pos] as i32;
    let mut res = bit & 0x3f;
    let sign = (bit & 0x40) == 0;

    while (bit & 0x80) != 0 {
        *pos += 1;
        bit = arr[*pos] as i32;
        res += (bit & 0x7f) << shl;
        shl += 7;
    }
    *pos += 1;

    if sign { res } else { -res }
}

pub fn vol_amount(ivol: i32) -> f64 {
    let logpoint = ivol >> 24;
    let hleax = (ivol >> 16) & 0xff;
    let lheax = (ivol >> 8) & 0xff;
    let lleax = ivol & 0xff;
    let dw_ecx = logpoint * 2 - 0x7f;
    let dw_edx = logpoint * 2 - 0x86;
    let dw_esi = logpoint * 2 - 0x8e;
    let dw_eax = logpoint * 2 - 0x96;

    let dbl_xmm6 = if dw_ecx < 0 { 1.0 / 2.0f64.powi(-dw_ecx) } else { 2.0f64.powi(dw_ecx) };

    let dbl_xmm4 = if hleax > 0x80 {
        2.0f64.powi(dw_edx) * 128.0 + (hleax & 0x7f) as f64 * 2.0f64.powi(dw_edx + 1)
    } else if dw_edx >= 0 {
        2.0f64.powi(dw_edx) * hleax as f64
    } else {
        (1.0 / 2.0f64.powi(dw_edx)) * hleax as f64
    };

    let (dbl_xmm3, dbl_xmm1) = if (hleax & 0x80) != 0 {
        (2.0f64.powi(dw_esi + 1) * lheax as f64, 2.0f64.powi(dw_eax + 1) * lleax as f64)
    } else {
        (2.0f64.powi(dw_esi) * lheax as f64, 2.0f64.powi(dw_eax) * lleax as f64)
    };

    // dbg!(dbl_xmm6, dbl_xmm4, dbl_xmm3, dbl_xmm1);
    dbl_xmm6 + dbl_xmm4 + dbl_xmm3 + dbl_xmm1
}

// def get_time(buffer, pos):
//     (tminutes, ) = struct.unpack("<H", buffer[pos: pos + 2])
//     hour = int(tminutes / 60)
//     minute = tminutes % 60
//     pos += 2
//
//     return hour, minute, pos

#[test]
fn check_datetime_price_vol_amount() {
    #[rustfmt::skip]
    assert_eq!(datetime(&[235, 100, 52, 1], 9),
               DateTime { year: 2021, month: 9, day: 23, hour: 15, minute: 0, });
    assert_eq!(price(&[180, 154, 2], &mut 0), 18100);
    assert_eq!(price(&[228, 6], &mut 0), -420);
    assert_eq!(price(&[156, 3], &mut 0), 220);
    assert_eq!(price(&[194, 7], &mut 0), -450);
    assert_eq!(vol_amount(1235775464), 1379837.0);
    assert_eq!(vol_amount(1326643033), 2465683712.0);

    // let arr = [235, 100, 52, 1, 180, 154, 2, 228, 6, 156, 3, 194, 7, 232, 111, 168, 73, 89,
    // 247, 18, 79];
}
