use eyre::{ContextCompat, Result, WrapErr};
use serde::{Deserialize, Deserializer, Serialize};

/// 获取股票数据
pub fn get(page_size: u16, page_number: u16) -> Result<String> {
    // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
    // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
    let url = format!(
        "https://push2.eastmoney.com/api/qt/clist/get?np=1&fltt=1&invt=2&\
        cb=jQuery37108306728141019732_1742893112795&fs=m%3A0%2Bt%3A6%2Cm%3A0%2Bt%3A80%2Cm%3A1%2Bt%3A2%2Cm%3A1%2Bt%3A23%2Cm%3A0%2Bt%3A81%2Bs%3A2048&\
        fields=f12%2Cf13%2Cf14%2Cf1%2Cf2%2Cf4%2Cf3%2Cf152%2Cf5%2Cf6%2Cf7%2Cf15%2Cf18%2Cf16%2Cf17%2Cf10%2Cf8%2Cf9%2Cf23&\
        fid=f12&pn={page_number}&pz={page_size}&po=1&dect=1&ut=fa5fd1943c7b386f172d6893dbfba10b&wbp2u=%7C0%7C0%7C0%7Cweb&_=1742893112836"
    );
    info!("Get: {url}");
    Ok(ureq::get(&url)
        .call()
        .wrap_err_with(|| format!("获取东财股票数据失败，网址为\n`{url:?}`"))?
        .into_string()?)
}

pub fn parse(text: &str) -> Result<EastMarket> {
    // jQuery112407375845698232317_1631693257414();
    let start = text
        .find('{')
        .wrap_err_with(|| format!("{text:?} 不含 JSON"))?;
    serde_json::from_str(&text[start..text.len() - 2])
        .wrap_err_with(|| format!("解析东财股票数据失败，返回的文本为\n{text:?}"))
}

/// 获取并解析股票数据
pub fn fetch(max: Option<u16>) -> Result<EastMarket> {
    // NOTE: 最多只能获取 100 条数据；page size 从第一页开始
    const N: u16 = 100;

    let mut done = 0u16;
    let mut total;
    let mut v = Vec::with_capacity(6000);
    loop {
        let page_number = done / 100 + 1;

        let txt = get(N, page_number)?;
        let data = parse(&txt)?;
        total = data.data.total;
        let len = data.data.diff.len();
        done += len as u16;
        v.push(data);
        info!("page_number={page_number} len={len} done={done} max={max:?} total={total}");
        if done >= max.unwrap_or(total) {
            break;
        }
    }
    let mut diff = Vec::with_capacity(total as usize);
    for each in v {
        let dtotal = each.data.total;
        // 保证所有响应的 total 一致
        ensure!(
            dtotal == total,
            "data.total {dtotal} should equal to {total}"
        );
        diff.extend(each.data.diff);
    }

    // 这里的数据保留了无效值（退市）的股票，因此严格检查数量
    if max.is_none() {
        ensure!(
            diff.len() as u16 == total,
            "Aggregated res.len() {} should equal to {total}",
            diff.len()
        );
    }

    Ok(EastMarket {
        data: EastData { diff, total },
    })
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
