use eyre::{anyhow, Result};
use std::collections::HashSet;

// 股票上限
const SH8: &str = "500";
const SH1: &str = "1800";

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
        .set("ACCEPT", "application/json, text/javascript, */*; q=0.01")
        .set("ACCEPT_LANGUAGE", ACCEPT_LANGUAGE)
        .set("CACHE_CONTROL", "no-cache")
        .set("CONNECTION", "keep-alive")
        .set("CONTENT_TYPE", "application/json")
        .set("DNT", "1")
        .set("PRAGMA", "no-cache")
        .set("REFERER", "http://www.szse.cn/market/trend/index.html")
        .set("USER_AGENT", USER_AGENT)
}
pub fn ureq_sh_with_headers(url: &str) -> ureq::Request {
    let cookie = "yfx_c_g_u_id_10000042=_ck21022514270615975949368753826;yfx_mr_10000042=\
            %3A%3Amarket_type_free_search%3A%3A%3A%3Abaidu%3A%3A%3A%3A%3A%3A%3A%3A\
            www.baidu.com%3A%3A%3A%3Apmf_from_free_search; yfx_key_10000042=;\
            VISITED_FUND_CODE=%5B%22501000%22%5D; yfx_mr_f_10000042\
            =%3A%3Amarket_type_free_search%3A%3A%3A%3Abaidu%3A%3A%3A%3A%3A%3A%3A%3A\
            www.baidu.com%3A%3A%3A%3Apmf_from_free_search;\
            VISITED_COMPANY_CODE=%5B%22501000%22%2C%22600000%22%2C%22600017%22%5D;\
            seecookie=%5B600000%5D%3A%u6D66%u53D1%u94F6%u884C%2C%5B600017%5D%3A%u65E5%u7167%u6E2F;\
            VISITED_STOCK_CODE=%5B%22600017%22%5D; VISITED_MENU\
            =%5B%228314%22%2C%228316%22%2C%228317%22%2C%228453\
            %22%2C%229062%22%2C%228529%22%2C%228530%22%2C%229055\
            %22%2C%228535%22%2C%228525%22%2C%228528%22%5D;\
            yfx_f_l_v_t_10000042=f_t_1614234426586__r_t_1630917175096__v_t_1630931359300__r_c_4";

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
    let url = format!("http://query.sse.com.cn/security/stock/getStockListData\
          .do?&jsonCallBack=jsonpCallback72491&isPagination=true&stockCode=&csrcCode=&areaName=\
          &stockType={stocktype}&pageHelp.cacheSize=1&pageHelp.beginPage=1&pageHelp.pageSize={pagesize}\
          &pageHelp.pageNo=2&pageHelp.endPage=21&_=1630931360678");
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
