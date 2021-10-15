use std::path::Path;

#[cfg(feature = "serde")]
use {
    crate::bytes_helper::{ser_code_string, ser_date_string},
    serde::{Serialize, Serializer},
};

pub mod fq;

/// 解析 `*.day` 文件中的一条日线数据，即其 32 个字节所代表的所有信息。
///
/// 注意：这个类型只对 `*.day` 文件进行了初步解析，
/// 所以日期 `date` 和股票代码 `code` 都是 `u32` 类型，
/// 如果的确需要这两个字段为字符串类型，考虑使用 [`serde_type::Day`] 类型。
///
/// [`serde_type::Day`]: crate::serde_type::Day
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Day {
    #[cfg_attr(feature = "serde", serde(serialize_with = "ser_date_string"))]
    pub date:   u32,
    #[cfg_attr(feature = "serde", serde(serialize_with = "ser_code_string"))]
    pub code:   u32,
    pub open:   f32,
    pub high:   f32,
    pub low:    f32,
    pub close:  f32,
    pub amount: f32,
    #[cfg_attr(feature = "serde", serde(serialize_with = "ser_vol"))]
    pub vol:    u32,
}

#[cfg(feature = "serde")]
fn ser_vol<S>(vol: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    serializer.serialize_f32(*vol as f32 / 100.)
}

impl Day {
    /// 从 `*.day` 文件中获取数据，该文件中，每 32 个字节存储了一根日线数据，
    /// 各字节存储数据如下：
    ///
    /// |   字节位置   |     含义     | 解析方式 |     额外处理     |
    /// | ------------ | ------------ | -------- | ---------------- |
    /// | 00 ~ 03 字节 | 年月日       | u32      | 见下文           |
    /// | 04 ~ 07 字节 | 开盘价       | u32      | 所解析的数字/100 |
    /// | 08 ~ 11 字节 | 最高价       | u32      | 所解析的数字/100 |
    /// | 12 ~ 15 字节 | 最低价       | u32      | 所解析的数字/100 |
    /// | 16 ~ 19 字节 | 收盘价       | u32      | 所解析的数字/100 |
    /// | 20 ~ 23 字节 | 成交额（元） | f32      | /                |
    /// | 24 ~ 27 字节 | 成交量（股） | u32      | /                |
    /// | 28 ~ 31 字节 | 保留字段     | /        | /                |
    ///
    /// 年月日处理方式：例如对 `20210810` 结果进行如下处理
    ///
    /// 年：20210810/10000 = 2021
    ///
    /// 月：20210810%10000/100 = 8
    ///
    /// 日：20210810%10000%100 = 10
    pub fn from_bytes(code: u32, arr: &[u8]) -> Self {
        use crate::bytes_helper::{f32_from_le_bytes, u32_from_le_bytes};
        Self { date: u32_from_le_bytes(arr, 0),
               open: u32_from_le_bytes(arr, 4) as f32 / 100.,
               high: u32_from_le_bytes(arr, 8) as f32 / 100.,
               low: u32_from_le_bytes(arr, 12) as f32 / 100.,
               close: u32_from_le_bytes(arr, 16) as f32 / 100.,
               amount: f32_from_le_bytes(arr, 20),
               vol: u32_from_le_bytes(arr, 24),
               code }
    }

    /// 一次性以**同步**方式读取单个 `*.day` 文件所有数据，然后转化成 Vec。
    pub fn from_file_into_vec<P: AsRef<Path>>(code: u32, p: P) -> crate::Result<Vec<Day>> {
        Ok(std::fs::read(p)?.chunks_exact(32).map(|b| Self::from_bytes(code, b)).collect())
    }

    // #[cfg(feature="tokio")]
    // pub async fn from_file_into_vec<P: AsRef<Path>>(code: u32, p: P) -> Result<Vec<Day>, Error>
    // {     Ok(tokio::fs::read(p).await.map_err(|_| Error::FileNotFound)?
    //         .chunks_exact(32)
    //         .map(|b| Self::from_bytes(code, b))
    //         .collect())
    // }

    // /// 转化成用于（反）序列化的数据类型：
    // /// 6 位字符串的股票代码；%Y-%m-%d 字符串格式的日期；f64 类型的成交额；u64 类型的 vol 。
    // #[cfg(feature = "serde")]
    // pub fn into_serde_type(self) -> crate::serde_type::Day {
    //     crate::serde_type::Day { code:   format!("{:06}", self.code),
    //                              date:   self.date_string(),
    //                              open:   self.open,
    //                              high:   self.high,
    //                              low:    self.low,
    //                              close:  self.close,
    //                              // 单位：元
    //                              amount: self.amount,
    //                              // 转换成手：方便与其他数据源汇合
    //                              vol:    self.vol as f32 / 100., }
    // }

    /// `%Y-%m-%d` 格式的日期
    pub fn date_string(&self) -> String { crate::bytes_helper::date_string(self.date) }

    /// `[年, 月, 日]` 格式的日期
    pub fn ymd_arr(&self) -> [u32; 3] {
        let x = self.date;
        [x / 10000, x % 10000 / 100, x % 10000 % 100]
    }

    /// chrono 格式的日期：用于某些序列化或者与时间相关的计算
    #[cfg(feature = "chrono")]
    pub fn ymd(&self) -> chrono::naive::NaiveDate {
        let [y, m, d] = self.ymd_arr();
        chrono::naive::NaiveDate::from_ymd(y as i32, m, d)
    }
}
