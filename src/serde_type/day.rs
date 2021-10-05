use serde::{Deserialize, Serialize};
use std::path::Path;

/// 用于（反）序列化：比如读取或写入到 csv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    /// `date` 为 `%Y-%m-%d` 文本格式
    pub date:   String,
    pub code:   String,
    pub open:   f32,
    pub high:   f32,
    pub low:    f32,
    pub close:  f32,
    pub amount: f32,
    pub vol:    f32,
}

impl Day {
    /// 参考 [`crate::file::day::Day::from_bytes`] 文档
    #[inline]
    pub fn from_bytes(code: u32, arr: &[u8]) -> Self {
        crate::file::day::Day::from_bytes(code, arr).into_serde_type()
    }

    /// 一次性读取单个 `*.day` 文件所有数据，然后转化成 Vec。
    pub fn from_file_into_vec<P: AsRef<Path>>(code: u32, p: P) -> crate::Result<Vec<Day>> {
        Ok(std::fs::read(p)?.chunks_exact(32).map(|b| Self::from_bytes(code, b)).collect())
    }
}
