use eyre::{anyhow, Result};
use std::collections::HashSet;

// 股票上限
const SH8: &str = "800";
const SH1: &str = "2000";

pub type StockList = HashSet<String>;

#[derive(Debug)]
pub struct SHSZ {
    sh1: usize,
    sh8: usize,
    sz: usize,
}

impl SHSZ {
    /// 计算总和
    pub fn count(&self) -> usize {
        let SHSZ { sh1, sh8, sz } = &self;
        sh1 + sh8 + sz
    }
}

/// 获取上证和深证的股票代码
///
/// 数量如下：SHSZ { sh1: 1646, sh8: 407, sz: 2745, } -> 4798
///
/// 见 `tests-integration::fetch_code::offical_stocks` 测试
pub fn offical_stocks(set: &mut StockList) -> Result<SHSZ> {
    let count = SHSZ {
        sh1: get_sh_stocks(set, "1", SH1)?,
        sh8: get_sh_stocks(set, "8", SH8)?,
        sz: get_sz_stocks(set)?,
    };
    info!("股票数量 {count:?}");
    Ok(count)
}

pub fn get_offical_stocks(cond: &str) -> Result<StockList> {
    let mut set = StockList::with_capacity(6000);
    let len = match cond {
        "official" => offical_stocks(&mut set)?.count(),
        "szse" => get_sz_stocks(&mut set)?,
        "sse" => get_sh_stocks(&mut set, "8", SH8)? + get_sh_stocks(&mut set, "1", SH1)?,
        _ => unreachable!("请输入 official | szse | sse 中的一个"),
    };

    info!("获得上证和深证股票数量：{len}");
    Ok(set)
}

/// 深交所官网的 A 股和创业板股票信息。
pub fn get_sz_stocks(set: &mut StockList) -> Result<usize> {
    use calamine::{DataType, Reader, Xlsx};
    use std::io::Read;
    let (url, ex) = (
        "http://www.szse.cn/api/report/ShowReport?\
        SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random=0.8587844061443386",
        "sz",
    );
    let bytes = &mut Vec::with_capacity(1 << 20);
    ureq::get(url).call()?.into_reader().read_to_end(bytes)?;
    let mut workbook = Xlsx::new(std::io::Cursor::new(bytes))?;
    // 每个单元格被解析的类型可能会不一样，所以把股票代码统一转化成字符型
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        set.extend(range.rows().skip(1).map(|r| match &r[4] {
            DataType::Int(x) => format!("{ex}{x}"),
            DataType::Float(x) => {
                format!("{}{}", ex, *x as i64)
            }
            DataType::String(x) => format!("{ex}{x}"),
            _ => unreachable!(),
        }));
        Ok(range.height() - 1)
    } else {
        Err(anyhow!("xlsx parse error"))
    }
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36";
const ACCEPT_LANGUAGE: &str =
    "zh-CN,zh;q=0.9,de;q=0.8,ko;q=0.7,ru;q=0.6,it;q=0.5,ga;q=0.4,en;q=0.3";

pub fn ureq_sz_with_headers(url: &str) -> ureq::Request {
    ureq::get(url)
        .set("ACCEPT", "*/*")
        .set("ACCEPT_LANGUAGE", ACCEPT_LANGUAGE)
        .set("CACHE_CONTROL", "no-cache")
        .set("CONNECTION", "keep-alive")
        .set("CONTENT_TYPE", "application/json")
        .set("DNT", "1")
        .set("PRAGMA", "no-cache")
        .set("REFERER", "http://www.sse.com.cn/")
        .set("USER_AGENT", USER_AGENT)
}
pub fn ureq_sh_with_headers(url: &str) -> ureq::Request {
    let cookie = "ba17301551dcbaf9_gdp_user_key=; \
                  ba17301551dcbaf9_gdp_session_id=0876b773-38a3-44d0-bb4a-2b5569025b82; \
                  gdp_user_id=gioenc-2g8894g6%2C764a%2C50d8%2C8d6g%2C3bg6194752ce; \
                  ba17301551dcbaf9_gdp_session_id_0876b773-38a3-44d0-bb4a-2b5569025b82=true; \
                  JSESSIONID=0311A5533F5FD798EE9DAFDE6A1D70A7; \
                  ba17301551dcbaf9_gdp_sequence_ids=\
                  {%22globalKey%22:14%2C%22VISIT%22:2%2C%22PAGE%22:5%2C%22VIEW_CHANGE%22:2%2C%22CUSTOM%22:3%2C%22VIEW_CLICK%22:6}";
    ureq::get(url)
        .set("ACCEPT", "*/*")
        .set("ACCEPT_LANGUAGE", ACCEPT_LANGUAGE)
        .set("CACHE_CONTROL", "no-cache")
        .set("CONNECTION", "keep-alive")
        .set("COOKIE", cookie)
        .set("PRAGMA", "no-cache")
        .set("REFERER", "http://www.sse.com.cn/")
        .set("USER_AGENT", USER_AGENT)
}

// 上交所 科创板 68 开头（目前 350 只，只需一次请求） => stockType=8, pagesize=400
//        A 股 60 开头（目前 1650 只，只需一次请求） => stockType=1, pagesize=1700
fn request_sh(stocktype: &str, pagesize: &str) -> ureq::Request {
    let url = format!(
        "http://query.sse.com.cn/sseQuery/commonQuery.do?\
        jsonCallBack=jsonpCallback37525685&STOCK_TYPE={stocktype}\
        &REG_PROVINCE=&CSRC_CODE=&STOCK_CODE=&sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L\
        &COMPANY_STATUS=2%2C4%2C5%2C7%2C8&type=inParams&isPagination=true\
        &pageHelp.cacheSize=1&pageHelp.beginPage=1&pageHelp.pageSize={pagesize}\
        &pageHelp.pageNo=1&pageHelp.endPage=1&_=1680491539414"
    );
    ureq_sh_with_headers(&url)
}

// 上交所 科创板 68 开头（目前 350 只，只需一次请求） => stockType=8, pagesize=400
//        A 股 60 开头（目前 1650 只，只需一次请求） => stockType=1, pagesize=1700
pub fn get_sh_stocks(set: &mut StockList, stocktype: &str, pagesize: &str) -> Result<usize> {
    let text = request_sh(stocktype, pagesize).call()?.into_string()?;
    let pos1 = text
        .find("total\":")
        .ok_or(anyhow!("`Total` field not found"))?
        + 7;
    let pos2 = text[pos1..pos1 + 10]
        .find('}')
        .ok_or(anyhow!("`Total` field not found"))?
        + pos1;
    let n: usize = text[pos1..pos2].parse()?;
    // 注意：如果不 take 的话，split 有一半是重复的

    set.extend(
        text.split("COMPANY_CODE")
            .skip(1)
            .take(n)
            .map(|s| format!("sh{}", &s[3..9])),
    );
    Ok(n)
}
