use anyhow::{anyhow, Result};
use argh::FromArgs;
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::io::{HEADER_SSE, HEADER_SZSE};

/// 获取交易所官网数据
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "official")]
pub struct Official {
    /// 交易所
    #[argh(option, short = 'e')]
    pub exchange: Option<String>,

    /// sh 交易所
    #[argh(option, short = 'k')]
    pub link: Option<String>,

    /// 以 json 格式显示
    #[argh(switch, short = 'j')]
    pub json: bool,

    /// 从第几条开始。默认为 0 。只对 sh 有效。
    #[argh(option, short = 'b', default = "0")]
    pub begin: u16,

    /// 从第几条结束。默认为 2100 。两个交易所目前股票数量的最大值。
    #[argh(option, short = 'n', default = "2100")]
    pub end: u16,

    /// 查询的开始日期。只对 sz 有效，因为 sh 不支持查询日期，每次都是当天数据。
    #[argh(option, short = 'd', default = r#""2021-09-15".into()"#)]
    pub begin_date: String,

    /// 查询的开始日期。只对 sz 有效，因为 sh 不支持查询日期，每次都是当天数据。
    /// 而且日期范围不超过三天（含首尾）。
    #[argh(option, short = 'D', default = r#""2021-09-15".into()"#)]
    pub end_date: String,
}

impl Official {
    // /// 每日市场行情
    // pub async fn daily_market(&self) -> Result<String> {
    //     let client = reqwest::Client::new();
    //     let url = match self.exchange.as_deref() {
    //         // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
    //         // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
    //         Some("sh") => format!("http://yunhq.sse.com.cn:32041//v1/sh1/list/exchange/equity?\
    //           callback=jQuery112406614406761214793_1631509021909&\
    //           select=code%2Cname%2Copen%2Chigh%2Clow%2Clast%2Cprev_close%2Cchg_rate%2C\
    //           volume%2Camount%2Cchange%2Camp_rate%2Ccpxxsubtype&\
    //           order=&begin={}&end={}&_=1631509021915", self.begin, self.end),
    //         Some("sz") => format!("http://www.szse.cn/api/report/ShowReport/data?\
    //           SHOWTYPE=JSON&CATALOGID=1815_stock&TABKEY=tab1&\
    //           PAGENO=1&tab1PAGESIZE={}&txtBeginDate=2021-09-13&txtEndDate=2021-09-13\
    //           &radioClass=00%2C20%2C30&txtSite=all&random=0.30236852035138306", self.end),
    //         _ => todo!(),
    //     };
    //     let text = tokio::spawn(client.get(url)
    //                                   .headers(HEADER_SSE.to_owned())
    //                                   .send()
    //                                   .await?
    //                                   .text()).await??;
    //     println!("{}", text);
    //     Ok(text)
    // }

    pub async fn sz_market(&self) -> Result<()> {
        let client = reqwest::Client::new();
        // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
        // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
        let url = format!("http://www.szse.cn/api/report/ShowReport/data?\
              SHOWTYPE=JSON&CATALOGID=1815_stock&TABKEY=tab1&\
              PAGENO=1&tab1PAGESIZE={}&txtBeginDate={}&txtEndDate={}\
              &radioClass=00%2C20%2C30&txtSite=all&random=0.30236852035138306", 
              self.end, self.begin_date, self.end_date);
        let text = tokio::spawn(client.get(url)
                                      .headers(HEADER_SZSE.to_owned())
                                      .send()
                                      .await?
                                      .text()).await??;
        println!("{text}");
        if self.json {
            let json: SzMarket = serde_json::from_str(&text[1..text.len() - 1])?;
            println!("{json:?}");
        }
        Ok(())
    }

    pub async fn sh_market(&self) -> Result<()> {
        let client = reqwest::Client::new();
        // 如果需要升序，使用 `order=code%2Case` 或者 `order=`
        // ashare => A 股，bshare => B 股，kshare => 科创板，equity => 前三种
        let url = format!("http://yunhq.sse.com.cn:32041//v1/sh1/list/exchange/equity?\
              callback=jQuery112406614406761214793_1631509021909&\
              select=code%2Cname%2Copen%2Chigh%2Clow%2Clast%2Cprev_close%2Cchg_rate%2C\
              volume%2Camount%2Cchange%2Camp_rate%2Ccpxxsubtype&\
              order=&begin={}&end={}&_=1631509021915", self.begin, self.end);
        let text = tokio::spawn(client.get(url)
                                      .headers(HEADER_SSE.to_owned())
                                      .send()
                                      .await?
                                      .text()).await??;
        println!("{text}");
        if self.json {
            // jQuery1124043835116035075705_1631539496628()
            let json: ShMarket = serde_json::from_str(&text[42..text.len() - 1])?;
            println!("{json:?}");
        }
        Ok(())
    }

    pub async fn sh(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let text = tokio::spawn(client.get(self.link.as_ref().ok_or(anyhow!("请检查网址！"))?)
                                      .headers(HEADER_SSE.to_owned())
                                      .send()
                                      .await?
                                      .text()).await??;
        println!("{text}");
        if self.json {
            // jQuery1124043835116035075705_1631539496628()
            let json: ShMarket = serde_json::from_str(&text[42..text.len() - 1])?;
            println!("{json:?}");
        }
        Ok(())
    }

    pub async fn sz(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let text = tokio::spawn(client.get(self.link.as_ref().ok_or(anyhow!("请检查网址！"))?)
                                      .headers(HEADER_SZSE.to_owned())
                                      .send()
                                      .await?
                                      .text()).await??;
        println!("{text}");
        if self.json {
            // jQuery1124043835116035075705_1631539496628()
            let json: ShMarket = serde_json::from_str(&text[42..text.len() - 1])?;
            println!("{json:?}");
        }
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        let task = async {
            match self.exchange.as_deref() {
                Some("sh") if self.link.is_some() => self.sh().await,
                Some("sh") => self.sh_market().await,
                Some("sz") if self.link.is_some() => self.sz().await,
                Some("sz") => self.sz_market().await,
                _ => todo!(),
            }
        };
        crate::io::RUNTIME.block_on(task)
    }
}

/// 上交所全市场最新行情表。
/// http://www.sse.com.cn/market/price/report/
#[derive(Debug, Serialize, Deserialize)]
pub struct ShMarket {
    begin: u16,
    date:  u32,
    end:   u16,
    list:  Vec<ShMarketTable>,
    time:  u32,
    total: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Category {
    ASH,
    BSH,
    KSH,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShMarketTable {
    /// 6 位数字的股票代码
    pub code:     String,
    /// 股票名称
    pub name:     String,
    /// 开盘价
    pub open:     f64,
    /// 最高价
    pub high:     f64,
    /// 最低价
    pub low:      f64,
    /// 最新价（收盘之后为收盘价）
    pub close:    f64,
    /// 昨收
    pub preclose: f64,
    /// 涨跌幅（收盘-昨收） / 昨收 * 100
    pub pct:      f64,
    /// 成交量（vol/100 的单位为手）
    pub vol:      u64,
    /// 成交额（元）
    pub amount:   f64,
    // /// 常量？ "E110"
    // pub e110:     String,
    /// 涨跌（收盘或者说最新价-昨收）
    pub change:   f64,
    /// 振幅（当日最高-当日最低） / 昨收 * 100
    pub vibr:     f64,
    /// 股票类型：`ASH` | `KSH` | `BSH`
    pub category: Category,
    /* /// 常量？ "   D  F             "
     * pub df:       String, */
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SzMarket {
    pub data:     Vec<SzMarketTable>,
    pub metadata: SzMetaData,
}

// /// cjgs: "成交量<br>(万)"
// /// cjje: "成交金额<br>(万元)"
// /// jyrq: "交易日期"
// /// ks: "开盘"
// /// qss: "前收"
// /// sdf: "涨跌幅<br>（%）"
// /// ss: "今收"
// /// syl1: "市盈率"
// /// zd: "最低"
// /// zg: "最高"
// /// zqdm: "证券代码"
// /// zqjc: "证券简称
#[derive(Debug, Serialize, Deserialize)]
pub struct SzMarketTable {
    #[serde(deserialize_with = "comma_str_to_float", rename = "cjgs")]
    pub vol:      f64,
    #[serde(deserialize_with = "comma_str_to_float", rename = "cjje")]
    pub amount:   f64,
    #[serde(rename = "jyrq")]
    pub date:     String,
    #[serde(deserialize_with = "str_to_float", rename = "ks")]
    pub open:     f64,
    #[serde(deserialize_with = "str_to_float", rename = "qss")]
    pub preclose: f64,
    #[serde(deserialize_with = "str_to_float", rename = "sdf")]
    pub pct:      f64,
    #[serde(deserialize_with = "str_to_float", rename = "ss")]
    pub close:    f64,
    #[serde(deserialize_with = "str_to_float")]
    pub syl1:     f64,
    #[serde(deserialize_with = "str_to_float", rename = "zd")]
    pub low:      f64,
    #[serde(deserialize_with = "str_to_float", rename = "zg")]
    pub high:     f64,
    #[serde(rename = "zqdm")]
    pub code:     String,
    #[serde(rename = "zqjc")]
    pub name:     String,
}

fn str_to_float<'de, D: Deserializer<'de>>(des: D) -> Result<f64, D::Error> {
    <&str>::deserialize(des)?.parse().map_err(de::Error::custom)
}

/// 注意，把单位改成了元。成交量和成交额数据具有误差，因为原始数据就是失真的。
fn comma_str_to_float<'de, D: Deserializer<'de>>(des: D) -> Result<f64, D::Error> {
    <&str>::deserialize(des)?.replace(',', "")
                             .parse()
                             .map_err(de::Error::custom)
                             .map(|d: f64| 10000. * d)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SzMetaData {
    /// 股票总数
    pub recordcount: u16,
    /// 当前页码
    pub pageno:      u16,
    /// 总页数
    pub pagecount:   u16,
    /// 副标题。时间："2021-09-13 到 2021-09-13"
    pub subname:     String,
}
