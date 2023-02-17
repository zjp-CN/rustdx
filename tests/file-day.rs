use serde::{Deserialize, Serialize};
use std::path::Path;

/// 用于（反）序列化：比如读取或写入到 csv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    /// `date` 为 `%Y-%m-%d` 文本格式
    pub date: String,
    pub code: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub amount: f32,
    pub vol: f32,
}

impl Day {
    #[inline]
    pub fn from_bytes(code: u32, arr: &[u8]) -> Self {
        use rustdx::file::day::Day as DayRaw;
        let DayRaw {
            code,
            date,
            open,
            high,
            low,
            close,
            amount,
            vol,
        } = DayRaw::from_bytes(code, arr);
        Self {
            code: format!("{code:06}"),
            date: rustdx::bytes_helper::date_string(date),
            open,
            high,
            low,
            close,
            // 单位：元
            amount,
            // 转换成手：方便与其他数据源汇合
            vol: vol as f32 / 100.,
        }
    }

    pub fn from_file_into_vec<P: AsRef<Path>>(code: u32, p: P) -> rustdx::Result<Vec<Day>> {
        Ok(std::fs::read(p)?
            .chunks_exact(32)
            .map(|b| Self::from_bytes(code, b))
            .collect())
    }
}

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[test]
fn day() -> Result<()> {
    let path = "assets/sz000001.day";
    let day1 = Day::from_file_into_vec(1, path)?;
    let day2 = write_to_csv(rustdx::file::day::Day::from_file_into_vec(1, path)?)?;
    // insta::assert_yaml_snapshot!("serde-type", day1);
    assert_eq!(write_to_csv(day1)?, day2);
    insta::assert_debug_snapshot!("serde-type-csv-string", day2);
    Ok(())
}

fn write_to_csv(day: Vec<impl Serialize>) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    day.into_iter().try_for_each(|d| wtr.serialize(d))?;
    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}
