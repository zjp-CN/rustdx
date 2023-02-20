use std::path::Path;

/// 解析 `*.lc` 文件中的一条日线数据，即其 32 个字节所代表的所有信息。
///
/// 注意：这个类型只对 `*.lc` 文件进行了初步解析，
/// 所以日期 `date` 和股票代码 `code` 都是 `u32` 类型，
#[derive(Debug, Clone, Copy)]
pub struct Lc {
    pub date: u16,
    pub min: u16,
    pub code: u32,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub amount: f32,
    pub vol: u32,
}

impl Lc {
    /// 从 `*.lc` 文件中获取数据，该文件中，每 32 个字节存储了一根分钟 K 线数据，
    /// 各字节存储数据如下：
    ///
    /// | 字节位置     | 含义         | 解析方式 | 额外处理 |
    /// | ------------ | ------------ | -------- | -------- |
    /// | 00 ~ 01 字节 | 年月日       | u16      | 见下文 x |
    /// | 02 ~ 04 字节 | 分钟数       | u16      | 见下文 y |
    /// | 04 ~ 07 字节 | 开盘价       | u32      | /        |
    /// | 08 ~ 11 字节 | 最高价       | u32      | /        |
    /// | 12 ~ 15 字节 | 最低价       | u32      | /        |
    /// | 16 ~ 19 字节 | 收盘价       | u32      | /        |
    /// | 20 ~ 23 字节 | 成交额（元） | f32      | /        |
    /// | 24 ~ 27 字节 | 成交量（股） | u32      | /        |
    /// | 28 ~ 31 字节 | 保留字段     | /        | /        |
    ///
    /// “年月日时分”处理方式：
    ///
    /// | 单位 | 计算过程          |
    /// | ---- | ----------------- |
    /// | 年   | x / 2048 + 2004   |
    /// | 月   | x / 2048 / 100    |
    /// | 日   | x / 2048 % 100    |
    /// | 时   | y / 60            |
    /// | 分   | y % 60            |
    pub fn from_bytes(code: u32, arr: &[u8]) -> Self {
        use crate::bytes_helper::{f32_from_le_bytes, u16_from_le_bytes, u32_from_le_bytes};
        Self {
            date: u16_from_le_bytes(arr, 0),
            min: u16_from_le_bytes(arr, 2),
            open: f32_from_le_bytes(arr, 4),
            high: f32_from_le_bytes(arr, 8),
            low: f32_from_le_bytes(arr, 12),
            close: f32_from_le_bytes(arr, 16),
            amount: f32_from_le_bytes(arr, 20),
            vol: u32_from_le_bytes(arr, 24),
            code,
        }
    }

    /// 一次性以**同步**方式读取单个 `*.lc` 文件所有数据，然后转化成 Vec。
    pub fn from_file_into_vec<P: AsRef<Path>>(code: u32, p: P) -> crate::Result<Vec<Lc>> {
        Ok(std::fs::read(p)?
            .chunks_exact(32)
            .map(|b| Self::from_bytes(code, b))
            .collect())
    }

    /// 转化成用于（反）序列化的数据类型：
    /// 6 位字符串的股票代码；%Y-%m-%d 字符串格式的日期；f64 类型的成交额；u64 类型的 vol 。
    pub fn into_serde_type(self) -> LcSerde {
        LcSerde {
            datetime: self.datetime_string(),
            code: format!("{:06}", self.code),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            amount: self.amount,
            vol: self.vol,
        }
    }

    pub fn datetime_string(&self) -> String {
        self.hm_arr()
            .iter()
            .fold(self.date_string(), |acc, &x| format!("{acc:02}:{x:02}"))
    }

    /// `%Y-%m-%d` 格式的日期
    pub fn date_string(&self) -> String {
        let [y, m, d] = self.ymd_arr();
        let fill = |x: u16| if x > 9 { "" } else { "0" };
        format!("{}-{}{}-{}{}", y, fill(m), m, fill(d), d)
    }

    /// `[年, 月, 日]` 格式的日期
    pub fn ymd_arr(&self) -> [u16; 3] {
        let x = self.date;
        [x / 2048 + 2004, x % 2048 / 100, x % 2048 % 100]
    }

    pub fn hm_arr(&self) -> [u16; 2] {
        [self.min / 60, self.min % 60]
    }

    /// chrono 格式的日期：用于某些序列化或者与时间相关的计算
    pub fn datetime(&self) -> chrono::naive::NaiveDateTime {
        use chrono::naive::NaiveDate;
        const ERR: &str = "日期格式不对";
        let [y, m, d] = self.ymd_arr();
        let [h, min] = self.hm_arr();
        NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
            .expect(ERR)
            .and_hms_opt(h as u32, min as u32, 0)
            .expect(ERR)
    }
}

/// 用于序列化：比如写入到 csv
///
/// 此结构体暂时待定，未来可能更改。
#[derive(Debug, Clone, serde::Serialize)]
pub struct LcSerde {
    /// `date` 为 `%Y-%m-%d H:M` 文本格式
    pub datetime: String,
    pub code: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub amount: f32,
    pub vol: u32,
}
