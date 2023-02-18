use anyhow::Result;
use argh::FromArgs;
use serde::{Deserialize, Serialize};

/// 东方财富当日 A 股数据。多数情况下使用 `rustdx east -p factor.csv` 即可。
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "east")]
pub struct East {
    /// 保存数据的 CSV 路径文件名。默认为当前路径下 eastmoney.csv 文件。
    #[argh(option, short = 'o', default = r#""eastmoney.csv".into()"#)]
    pub output: String,

    /// 指定前一日复权 csv 文件。
    #[argh(option, short = 'p')]
    pub previous: Option<std::path::PathBuf>,

    /// debug 版本时，为了通过断言测试而手动忽略掉前收价格或复权因子计算不一致的股票。
    /// 这种不一致的原因：浮点误差、或者当天为除权日。当第一个元素为 `all` 时，直接通过测试。
    /// release 版本无需此选项。
    #[argh(option, short = 'i')]
    pub ignore: Vec<String>,

    /// 股票总个数，默认 4800。
    #[argh(option, short = 'n', default = "4800")]
    pub n: u16,

    /// 以 json 格式显示
    #[argh(switch, short = 'j')]
    pub json: bool,

    /// 打印响应数据。
    #[argh(switch, short = 's')]
    pub show: bool,

    /// 可选。指定表名称，默认为 `rustdx.tmp`。
    #[argh(option, short = 't', default = "String::from(\"rustdx.tmp\")")]
    pub table: String,
}

impl East {
    pub async fn get(&self) -> Result<String> {
        let client = reqwest::Client::new();
        // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
        // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
        let url = format!(
            "http://56.push2.eastmoney.com/api/qt/clist/get?\
            cb=jQuery112407375845698232317_1631693257414&\
            pn=1&pz={}&po=0&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&\
            fltt=2&invt=2&fid=f12&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23&\
            fields=f18,f16,f12,f17,f15,f2,f6,f5&_=1631693257534",
            self.n
        );
        let text = tokio::spawn(
            client
                .get(url)
                // .headers(HEADER_SSE.to_owned())
                .send()
                .await?
                .text(),
        )
        .await??;
        Ok(text)
    }

    pub fn json(text: &str) -> Result<EastMarket> {
        // jQuery112407375845698232317_1631693257414();
        let json: EastMarket = serde_json::from_str(&text[42..text.len() - 2])?;
        Ok(json)
    }

    /// 注意：即使没有提供前一天的 factor 数据，
    /// 产生的 csv 文件依然会有 factor 一列，但数据是 0.
    pub fn run_no_previous(&self) -> Result<()> {
        let text = crate::io::RUNTIME.block_on(self.get())?;
        let json = Self::json(&text)?;
        if self.show {
            println!("text:\n{text}\njson:\n{json:?}");
        }

        {
            let file = std::fs::File::create(&self.output)?;
            let mut wrt = csv::Writer::from_writer(file);
            for row in &json.data.diff {
                if let F32::Yes(_) = row.close {
                    wrt.serialize(row)?;
                }
            }
        }
        self.insert_clickhouse()
    }

    pub fn run_previous(&self) -> Result<()> {
        let text = crate::io::RUNTIME.block_on(self.get())?;
        let json = Self::json(&text)?;
        if self.show {
            println!("text:\n{text}\njson:\n{json:?}");
        }
        self._run_previous(json)?;
        self.insert_clickhouse()
    }

    fn _run_previous(&self, mut json: EastMarket) -> Result<()> {
        let previous = crate::io::previous_csv_table(&self.previous, &self.table)?;
        let file = std::fs::File::create(&self.output)?;
        let mut wrt = csv::Writer::from_writer(file);
        for row in &mut json.data.diff {
            // 排除掉无数据的股票：停牌、未上市之类
            if let (&F32::Yes(c), &F32::Yes(p)) = (&row.close, &row.preclose) {
                if let Some(f) = previous.get(&row.code.parse()?) {
                    row.factor = c as f64 / p as f64 * f.factor;
                    // 1. 由于数据源不同导致有误差，无法比较相等；
                    // 2. 当今天为除权除息日时，两边的 preclose 是不想等的，所以此时无法校验
                    // debug_assert_eq!(row.factor, f.compute_factor(c as f64));
                    #[cfg(debug_assertions)]
                    {
                        assert!(
                            if self.ignore.get(0).map(|x| x == "all").unwrap_or(false) {
                                true
                            } else if (p as f64 - f.preclose).abs() < 0.01 {
                                (row.factor - f.compute_factor(c as f64)).abs() < 0.01
                            } else {
                                self.ignore.iter().any(|x| x == row.code.as_str())
                            },
                            "code: #{}#\neast: factor: {}, preclose: {}\nfq: factor: {}, \
                                 preclose: {}",
                            row.code,
                            row.factor,
                            p,
                            f.compute_factor(c as f64),
                            f.preclose
                        );
                    }
                } else {
                    row.factor = c as f64 / p as f64;
                }
                wrt.serialize(row)?;
            }
        }
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        if self.previous.is_some() {
            self.run_previous()
        } else {
            self.run_no_previous()
        }
    }

    fn insert_clickhouse(&self) -> Result<()> {
        if self.output.eq("clickhouse") {
            // 插入 clickhouse 时，不保存解析结果。
            // 如果需要保存结果，则使用 `rustdx east -o output.csv` 指定 csv 文件。
            crate::io::insert_clickhouse(&self.output, &self.table, false)
        } else {
            Ok(())
        }
    }
}

/// 用于（反）序列化：比如读取东方财富网页返回的 json ；把结果写入到 csv
/// 注意：factor 需要提供前一天的 factor 数据才会计算（即 -p xx.csv）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day<'a> {
    /// `date` 为 `%Y-%m-%d` 文本格式
    #[serde(skip_deserializing, default = "default_date")]
    pub date: String,
    #[serde(rename(deserialize = "f12"))]
    pub code: String,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f17"))]
    pub open: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f15"))]
    pub high: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f16"))]
    pub low: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f2"))]
    pub close: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f6"))]
    pub amount: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f5"))]
    pub vol: F32<'a>,
    #[serde(borrow)]
    #[serde(rename(deserialize = "f18"))]
    pub preclose: F32<'a>,
    #[serde(skip_deserializing, default)]
    pub factor: f64,
}

/// 排除掉 "-" 无实际数据的股票
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum F32<'a> {
    Null(&'a str),
    Yes(f32),
}

/// TODO： 最新的交易日，而不是当天
fn default_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastMarket<'a> {
    #[serde(borrow)]
    pub data: EastData<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EastData<'a> {
    // pub diff:  Vec<Factor>,
    #[serde(borrow)]
    pub diff: Vec<Day<'a>>,
    pub total: u16,
}
