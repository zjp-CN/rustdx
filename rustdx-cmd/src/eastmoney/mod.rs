use eyre::{Result, WrapErr};
use serde::{Deserialize, Deserializer, Serialize};

/// 获取股票数据
pub fn get(n: u16) -> Result<String> {
    // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
    // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
    let url = format!(
        "http://56.push2.eastmoney.com/api/qt/clist/get?\
            cb=jQuery112407375845698232317_1631693257414&\
            pn=1&pz={n}&po=0&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&\
            fltt=2&invt=2&fid=f12&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23&\
            fields=f18,f16,f12,f17,f15,f2,f6,f5&_=1631693257534",
    );
    Ok(ureq::get(&url)
        .call()
        .wrap_err_with(|| format!("获取东财股票数据失败，网址为\n`{url:?}`"))?
        .into_string()?)
}

pub fn parse(text: &str) -> Result<EastMarket> {
    // jQuery112407375845698232317_1631693257414();
    serde_json::from_str(&text[42..text.len() - 2])
        .wrap_err_with(|| format!("解析东财股票数据失败，返回的文本为\n{text:?}"))
}

/// 获取并解析股票数据
pub fn fetch() -> Result<EastMarket> {
    // NOTE: 这与命令行默认的东财股票数量应该一致
    const N: u16 = 6000;
    let s = get(N)?;
    parse(&s)
}

/// 用于（反）序列化：比如读取东方财富网页返回的 json ；把结果写入到 csv
/// 注意：factor 需要提供前一天的 factor 数据才会计算（即 -p xx.csv）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    /// `date` 为 `%Y-%m-%d` 文本格式
    #[serde(skip_deserializing, default = "default_date")]
    pub date: String,
    #[serde(rename(deserialize = "f12"))]
    pub code: String,
    #[serde(rename(deserialize = "f17"), deserialize_with = "deser_opt_f32")]
    pub open: F32,
    #[serde(rename(deserialize = "f15"), deserialize_with = "deser_opt_f32")]
    pub high: F32,
    #[serde(rename(deserialize = "f16"), deserialize_with = "deser_opt_f32")]
    pub low: F32,
    #[serde(rename(deserialize = "f2"), deserialize_with = "deser_opt_f32")]
    pub close: F32,
    #[serde(rename(deserialize = "f6"), deserialize_with = "deser_opt_f32")]
    pub amount: F32,
    #[serde(rename(deserialize = "f5"), deserialize_with = "deser_opt_f32")]
    pub vol: F32,
    #[serde(rename(deserialize = "f18"), deserialize_with = "deser_opt_f32")]
    pub preclose: F32,
    #[serde(skip_deserializing, default)]
    pub factor: f64,
}

fn deser_opt_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<F32, D::Error> {
    Ok(f32::deserialize(deserializer).ok())
}

/// 排除掉 "-" 无实际数据的股票（完全可以不必考虑这些无效数据）
pub type F32 = Option<f32>;

/// TODO： 最新的交易日，而不是当天
fn default_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastMarket {
    pub data: EastData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastData {
    // pub diff:  Vec<Factor>,
    pub diff: Vec<Day>,
    pub total: u16,
}
