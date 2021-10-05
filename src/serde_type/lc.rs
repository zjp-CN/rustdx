use serde::{Deserialize, Serialize};

/// 用于（反）序列化：比如读取或写入到 csv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lc {
    /// `date` 为 `%Y-%m-%d H:M` 文本格式
    pub datetime: String,
    pub code:     String,
    pub open:     f32,
    pub high:     f32,
    pub low:      f32,
    pub close:    f32,
    pub amount:   f64,
    pub vol:      u64,
}
