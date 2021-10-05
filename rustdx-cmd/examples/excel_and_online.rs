use anyhow::{anyhow, Result};

/// sh8: 334
/// ["sh688001", "sh688002", "sh688003", "sh688004", "sh688005", "sh688006", "sh688007",
///  "sh688008", "sh688009", "sh688010"]
/// ["sh688787", "sh688788", "sh688789", "sh688793", "sh688798", "sh688799", "sh688800",
///  "sh688819", "sh688981", "sh689009"]
/// sh1: 1639
/// ["sh600000", "sh600004", "sh600006", "sh600007", "sh600008", "sh600009", "sh600010",
///  "sh600011", "sh600012", "sh600015"]
/// ["sh605398", "sh605399", "sh605488", "sh605499", "sh605500", "sh605507", "sh605577",
///  "sh605580", "sh605588", "sh605589"]
/// sz: 2488
/// ["sz000001", "sz000002", "sz000004", "sz000005", "sz000006", "sz000007", "sz000008",
///  "sz000009", "sz000010", "sz000011"]
/// ["sz301045", "sz301046", "sz301047", "sz301048", "sz301049", "sz301050", "sz301051",
///  "sz301052", "sz301053", "sz301055"]
fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let (sh8, sh1, sz) =
        rt.block_on(futures::future::try_join3(get_sh("8", "400"), get_sh("1", "1700"), get_sz()))?;
    // let (sh8, sh1) = rt.block_on(futures::future::try_join(get_sh("8", "20"), get_sh("1",
    // "20")))?;
    println!("sh8: {}\n{:?}\n{:?}", sh8.len(), &sh8[..10], &sh8[sh8.len() - 10..]);
    println!("sh1: {}\n{:?}\n{:?}", sh1.len(), &sh1[..10], &sh1[sh1.len() - 10..]);
    println!("sz: {}\n{:?}\n{:?}", sz.len(), &sz[..10], &sz[sz.len() - 10..]);

    Ok(())
}

// 上交所 科创板 68 开头（目前 350 只，只需一次请求） => stockType=8, pagesize=400
//        A 股 60 开头（目前 1650 只，只需一次请求） => stockType=1, pagesize=1700
async fn get_sh(stocktype: &str, pagesize: &str) -> Result<Vec<String>> {
    use reqwest::header::{
        HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, COOKIE, PRAGMA,
        REFERER, USER_AGENT,
    };
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::with_capacity(10);
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(USER_AGENT,
                   HeaderValue::from_static("Mozilla/5.0 (Windows NT 6.1; Win64; x64) \
                                             AppleWebKit/537.36 (KHTML, like Gecko) \
                                             Chrome/92.0.4515.107 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_static("http://www.sse.com.cn/"));
    headers.insert(ACCEPT_LANGUAGE,
                   HeaderValue::from_static("zh-CN,zh;q=0.9,de;q=0.8,ko;q=0.7,ru;q=0.6,it;q=0.5,\
                                             ga;q=0.4,en;q=0.3"));
    headers.insert(COOKIE, HeaderValue::from_static(
            "yfx_c_g_u_id_10000042=_ck21022514270615975949368753826;yfx_mr_10000042=\
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
            yfx_f_l_v_t_10000042=f_t_1614234426586__r_t_1630917175096__v_t_1630931359300__r_c_4"
            ));

    let url = format!("http://query.sse.com.cn/security/stock/getStockListData\
          .do?&jsonCallBack=jsonpCallback72491&isPagination=true&stockCode=&csrcCode=&areaName=\
          &stockType={}&pageHelp.cacheSize=1&pageHelp.beginPage=1&pageHelp.pageSize={}\
          &pageHelp.pageNo=2&pageHelp.endPage=21&_=1630931360678", stocktype, pagesize);
    let text = client.get(url).headers(headers).send().await?.text().await?;
    let pos1 = text.find("total\":").ok_or(anyhow!("`Total` field not found"))? + 7;
    let pos2 = text[pos1..pos1 + 10].find("}").ok_or(anyhow!("`Total` field not found"))? + pos1;
    let n: usize = text[pos1..pos2].parse()?;
    // 注意：如果不 take 的话，split 有一半是重复的
    Ok(text.split("COMPANY_CODE")
           .skip(1)
           .take(n)
           .map(|s| format!("sh{}", &s[3..9]))
           .collect())
}

/// 深交所官网的 A 股和创业板股票信息。
async fn get_sz() -> Result<Vec<String>> {
    use calamine::{DataType, Reader, Xlsx};
    let (url, prefix) = ("http://www.szse.cn/api/report/ShowReport?\
        SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random=0.8587844061443386", "sz");
    let bytes = reqwest::get(url).await?.bytes().await?;
    let mut workbook = Xlsx::new(std::io::Cursor::new(bytes))?;
    // 每个单元格被解析的类型可能会不一样，所以把股票代码统一转化成字符型
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        Ok(range.rows()
                .skip(1)
                .map(|r| match &r[4] {
                    DataType::Int(x) => format! {"{}{}", prefix, x.to_string()}.into(),
                    DataType::Float(x) => format! {"{}{}", prefix, *x as i64}.into(),
                    DataType::String(x) => format! {"{}{}", prefix, x}.into(),
                    _ => unreachable!(),
                })
                .collect())
    } else {
        Err(anyhow!("xlsx parse error"))
    }
}
