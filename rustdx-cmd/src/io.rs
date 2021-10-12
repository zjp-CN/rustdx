use crate::cmd::{DayCmd, Stocklist};
use anyhow::{anyhow, Result};
use rustdx::file::{
    day::fq::Day,
    gbbq::{Factor, Gbbq},
};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

const BUFFER_SIZE: usize = 32 * (1 << 20); // 32M

/// TODO 协程解析、异步缓冲写入（利用多核优势）
pub fn run_csv(cmd: &DayCmd) -> Result<()> {
    let hm = cmd.stocklist();
    let file = File::create(&cmd.output)?;
    let mut wtr = csv::WriterBuilder::new().buffer_capacity(BUFFER_SIZE).from_writer(file);
    for dir in &cmd.path {
        let n = filter_file(dir)?.count();
        println!("dir: {:?} day 文件数量：{}", dir, n);
        let take = cmd.amount.unwrap_or(n);

        let mut count: usize = 0;
        filter_file(dir)?.map(|f| (cmd.filter_ec(f.to_str().unwrap()), f))
                         .filter(|((b, _), s)| filter(*b, &s, hm.as_ref(), dir).unwrap_or(false))
                         .take(take)
                         .filter_map(|((_, code), src)| {
                             count += 1;
                             println!("#{:06}# {:?}", code, src);
                             rustdx::serde_type::Day::from_file_into_vec(code, src).ok()
                         })
                         .flatten()
                         .try_for_each(|t| wtr.serialize(t))?;

        print(dir, count, take);
    }
    wtr.flush().map_err(|e| e.into())
}

/// TODO 协程解析、异步缓冲写入（利用多核优势）
pub fn run_csv_fq(cmd: &DayCmd) -> Result<()> {
    // 股本变迁
    let mut bytes = fs::read(cmd.gbbq.as_ref().unwrap())?;
    let gbbq = Gbbq::filter_hashmap(Gbbq::iter(&mut bytes[4..]));

    // 股票列表
    let hm = cmd.stocklist();

    let file = File::create(&cmd.output)?;
    let mut wtr = csv::WriterBuilder::new().buffer_capacity(BUFFER_SIZE).from_writer(file);
    for dir in &cmd.path {
        let n = filter_file(dir)?.count();
        println!("dir: {:?} day 文件数量：{}", dir, n);
        let take = cmd.amount.unwrap_or(n);

        let mut count: usize = 0;
        filter_file(dir)?.map(|f| (cmd.filter_ec(f.to_str().unwrap()), f))
                         .filter(|((b, _), s)| filter(*b, &s, hm.as_ref(), dir).unwrap_or(false))
                         .take(take)
                         .filter_map(|((_, code), src)| {
                             count += 1;
                             println!("#{:06}# {:?}", code, src);
                             Day::new(code, src, gbbq.get(&code).map(Vec::as_slice)).ok()
                         })
                         .flatten()
                         .try_for_each(|t| wtr.serialize(t))?;

        print(dir, count, take);
    }
    wtr.flush().map_err(|e| e.into())
}

/// TODO 协程解析、异步缓冲写入（利用多核优势）
pub fn run_csv_fq_previous(cmd: &DayCmd) -> Result<()> {
    // 股本变迁
    let mut bytes = fs::read(cmd.gbbq.as_ref().unwrap())?;
    let gbbq = Gbbq::filter_hashmap(Gbbq::iter(&mut bytes[4..]));

    // 前收
    let previous = previous_csv_table(&cmd.previous, &cmd.table)?;

    // 股票列表
    let hm = cmd.stocklist();

    let file = File::create(&cmd.output)?;
    let mut wtr = csv::WriterBuilder::new().buffer_capacity(BUFFER_SIZE).from_writer(file);
    for dir in &cmd.path {
        let n = filter_file(dir)?.count();
        println!("dir: {:?} day 文件数量：{}", dir, n);
        let take = cmd.amount.unwrap_or(n);

        let mut count: usize = 0;
        filter_file(dir)?.map(|f| (cmd.filter_ec(f.to_str().unwrap()), f))
                         .filter(|((b, _), s)| filter(*b, &s, hm.as_ref(), dir).unwrap_or(false))
                         .take(take)
                         .filter_map(|((_, code), src)| {
                             count += 1;
                             println!("#{:06}# {:?}", code, src);
                             Day::concat(code,
                                         src,
                                         // 无分红数据并不意味着无复权数据
                                         gbbq.get(&code).map(Vec::as_slice),
                                         previous.get(&code)).ok()
                         })
                         .flatten()
                         .try_for_each(|t| wtr.serialize(t))?;

        print(dir, count, take);
    }
    wtr.flush().map_err(|e| e.into())
}

/// 筛选 day 文件
#[rustfmt::skip]
fn filter_file(dir: &Path) -> Result<impl Iterator<Item = std::path:: PathBuf>> {
    Ok(dir.read_dir()?
          .filter_map(|e| e.map(|f| f.path()).ok())
          .filter(|p| p.extension().map(|s| s == "day").unwrap_or_default()))
}

/// 筛选存在于股票列表的股票
#[inline]
fn filter(b: bool, src: &Path, hm: Option<&Stocklist>, dir: &Path) -> Option<bool> {
    let src = src.strip_prefix(dir).ok()?.to_str()?.strip_suffix(".day")?;
    Some(b && hm.map(|m| m.contains(src)).unwrap_or(true))
}

fn print(dir: &Path, count: usize, take: usize) {
    if count == 0 && take != 0 {
        println!("{:?} 目录下无 `.day` 文件符合要求", dir);
    } else if take == 0 {
        println!("请输入大于 0 的文件数量");
    } else {
        println!("{:?}\t已完成：{}", dir, count);
    }
}

fn database_table(table: &str) -> (&str, &str) {
    let pos = table.find('.').unwrap();
    table.split_at(pos) // (database_name, table_name)
}

fn setup_clickhouse(fq: bool, table: &str) -> Result<()> {
    let create_database = format!("CREATE DATABASE IF NOT EXISTS {}", database_table(table).0);
    let output = Command::new("clickhouse-client").args(["--query", &create_database]).output()?;
    check_output(output);
    #[rustfmt::skip]
    let create_table = if fq {
        format!("
            CREATE TABLE IF NOT EXISTS {}
            (
                `date` Date CODEC(DoubleDelta),
                `code` FixedString(6),
                `open` Float32,
                `high` Float32,
                `low` Float32,
                `close` Float32,
                `amount` Float64,
                `vol` Float64,
                `preclose` Float64,
                `factor` Float64
            )
            ENGINE = MergeTree()
            ORDER BY (date, code)
        ", table)
    } else {
        format!("
            CREATE TABLE IF NOT EXISTS {}
            (
                `date` Date CODEC(DoubleDelta),
                `code` FixedString(6),
                `open` Float32,
                `high` Float32,
                `low` Float32,
                `close` Float32,
                `amount` Float64,
                `vol` Float64
            )
            ENGINE = MergeTree()
            ORDER BY (date, code)
        ", table)
    }; // PARTITION BY 部分可能需要去掉
    let output = Command::new("clickhouse-client").args(["--query", &create_table]).output()?;
    check_output(output);
    Ok(())
}

/// clickhouse-client --query "INSERT INTO table FORMAT CSVWithNames" < clickhouse[.csv]
pub fn run_clickhouse(cmd: &DayCmd) -> Result<()> {
    use subprocess::{Exec, Redirection};
    setup_clickhouse(cmd.gbbq.is_some(), &cmd.table)?;
    cmd.run_csv()?;
    let file = File::open(&cmd.output)?;
    let query = format!("INSERT INTO {} FORMAT CSVWithNames", cmd.table);
    let capture = Exec::cmd("clickhouse-client").args(&["--query", &query])
                                                .stdin(Redirection::File(file))
                                                .capture()?;
    println!("stdout:\n{}stderr:\n{}", capture.stdout_str(), capture.stderr_str());
    assert!(capture.success());
    keep_csv(&cmd.output, cmd.keep_csv)?;
    Ok(())
}

type Previous = Result<std::collections::HashMap<u32, Factor>>;

pub fn previous_csv_table(path: &Option<std::path::PathBuf>, table: &str) -> Previous {
    if let Some(Some(path)) = path.as_ref().map(|p| p.to_str()) {
        if path == "clickhouse" {
            clickhouse_factor_csv(table)
        } else {
            previous_csv(path)
        }
    } else {
        Err(anyhow!("请检查 gbbq 路径"))
    }
}

/// 读取前收盘价（前 factor ）数据
pub fn previous_csv(p: impl AsRef<Path>) -> Previous {
    Ok(csv::Reader::from_reader(File::open(p)?).deserialize::<Factor>()
                                               .filter_map(|f| f.ok())
                                               .map(|f| (f.code.parse().unwrap(), f))
                                               .collect())
}

/// 获取当前最新 factor
fn clickhouse_factor_csv(table: &str) -> Previous {
    let args =
        ["--query",
         &format!("WITH (
                        SELECT max(date) AS date
                        FROM {0}
                        ) AS latest
                    SELECT
                        date,
                        code,
                        close,
                        factor
                    FROM {0}
                    WHERE latest = date
                    INTO OUTFILE 'factor.csv'
                    FORMAT CSVWithNames;",
                  table)];
    let output = Command::new("clickhouse-client").args(args).output()?;
    check_output(output);
    previous_csv("factor.csv")
}

/// TODO: 与数据库有关的，把库名、表名可配置
pub fn run_mongodb(cmd: &DayCmd) -> Result<()> {
    cmd.run_csv()?;
    // TODO:排查为什么 date 列无法变成 date 类型 date.date(2006-01-02)
    let (database_name, table_name) = database_table(&cmd.table);
    let args = ["--db",
                database_name,
                "--collection",
                table_name,
                "--type=csv",
                "--file",
                &cmd.output,
                "--columnsHaveTypes",
                "--fields=code.string()"];
    let output = Command::new("mongoimport").args(args).output()?;
    check_output(output);
    keep_csv(&cmd.output, cmd.keep_csv)?;
    Ok(())
}

fn check_output(output: std::process::Output) {
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}

fn keep_csv(fname: &str, keep: bool) -> io::Result<()> {
    if keep {
        fs::rename(fname, format!("{}.csv", fname))
    } else {
        fs::remove_file(fname)
    }
}

/// 读取本地 xls(x) 文件
pub fn read_xlsx(path: &str, col: usize, prefix: &str) -> Option<Stocklist> {
    use calamine::{open_workbook_auto, DataType, Reader};
    let mut workbook = open_workbook_auto(&path).ok()?;
    let format_ = |x: &str| format!("{}{}", crate::cmd::auto_prefix(prefix, x), x);
    // 每个单元格被解析的类型可能会不一样，所以把股票代码统一转化成字符型
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        Some(range.rows()
                  .skip(1)
                  .map(|r| match &r[col] {
                      DataType::Int(x) => format_(&x.to_string()),
                      DataType::Float(x) => format_(&(*x as i64).to_string()),
                      DataType::String(x) => format_(x),
                      _ => unreachable!(),
                  })
                  .collect())
    } else {
        None
    }
}

#[derive(Debug)]
pub struct StockList(Mutex<Stocklist>);

#[allow(dead_code)]
impl StockList {
    pub fn extend(&self, iter: impl IntoIterator<Item = String>) {
        self.0.lock().unwrap().extend(iter)
    }

    pub fn new() -> Self { Self(Mutex::new(Stocklist::new())) }

    pub fn with_capacity(n: usize) -> Self { Self(Mutex::new(Stocklist::with_capacity(n))) }

    pub fn len(&self) -> usize { self.0.lock().unwrap().len() }

    pub fn into_inner(self) -> Stocklist { self.0.into_inner().unwrap() }
}

/// sh8: 334
/// ["sh688001", "sh688002", ..., "sh688981", "sh689009"]
/// sh1: 1639
/// ["sh600000", "sh600004", ..., "sh605588", "sh605589"]
/// sz: 2488
/// ["sz000001", "sz000002", ..., "sz301053", "sz301055"]
pub async fn offical_stocks(set: Arc<StockList>) -> Result<usize> {
    let len = futures::try_join!(get_sh_stocks(set.clone(), "8", "400"),
                                 get_sh_stocks(set.clone(), "1", "1700"),
                                 get_sz_stocks(set.clone()))?;
    dbg!(len);
    let len = len.0 + len.1 + len.2;
    debug_assert_eq!(set.len(), len);
    Ok(len)
}

pub fn get_offical_stocks(cond: &str) -> Option<Stocklist> {
    let set = Arc::new(StockList::with_capacity(4816));
    let len = RUNTIME.block_on(async {
                         match cond {
                             "official" => offical_stocks(set.clone()).await,
                             "szse" => get_sz_stocks(set.clone()).await,
                             "sse" => {
                                 let l =
                                     futures::try_join!(get_sh_stocks(set.clone(), "8", "400"),
                                                        get_sh_stocks(set.clone(), "1", "1700"),)?;
                                 Ok(l.0 + l.1)
                             }
                             _ => unreachable!(),
                         }
                     })
                     .ok()?;

    dbg!(len);
    let set = Arc::try_unwrap(set).expect("获取交易所股票列表数据失败").into_inner();
    Some(set)
}

#[test]
fn test_get_offical_stocks() -> Result<()> {
    let set = Arc::new(StockList::with_capacity(4816));
    let len = RUNTIME.block_on(offical_stocks(set.clone()))?;
    let set = Arc::try_unwrap(set).expect("获取交易所股票列表数据失败").into_inner();
    assert_eq!(set.len(), len);
    dbg!(len);
    Ok(())
}

/// 深交所官网的 A 股和创业板股票信息。
pub async fn get_sz_stocks(set: Arc<StockList>) -> Result<usize> {
    use calamine::{DataType, Reader, Xlsx};
    let (url, ex) = ("http://www.szse.cn/api/report/ShowReport?\
        SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random=0.8587844061443386", "sz");
    let bytes = tokio::spawn(reqwest::get(url).await?.bytes()).await??;
    let mut workbook = Xlsx::new(io::Cursor::new(bytes))?;
    // 每个单元格被解析的类型可能会不一样，所以把股票代码统一转化成字符型
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        set.extend(range.rows().skip(1).map(|r| match &r[4] {
                                           DataType::Int(x) => format!("{}{}", ex, x),
                                           DataType::Float(x) => {
                                               format!("{}{}", ex, *x as i64)
                                           }
                                           DataType::String(x) => format!("{}{}", ex, x),
                                           _ => unreachable!(),
                                       }));
        Ok(range.height() - 1)
    } else {
        Err(anyhow!("xlsx parse error"))
    }
}

/// 上交所 科创板 68 开头（目前 350 只，只需一次请求） => stockType=8, pagesize=400
///        A 股 60 开头（目前 1650 只，只需一次请求） => stockType=1, pagesize=1700
pub async fn get_sh_stocks(set: Arc<StockList>, stocktype: &str, pagesize: &str) -> Result<usize> {
    let client = reqwest::Client::new();
    let url = format!("http://query.sse.com.cn/security/stock/getStockListData\
          .do?&jsonCallBack=jsonpCallback72491&isPagination=true&stockCode=&csrcCode=&areaName=\
          &stockType={}&pageHelp.cacheSize=1&pageHelp.beginPage=1&pageHelp.pageSize={}\
          &pageHelp.pageNo=2&pageHelp.endPage=21&_=1630931360678", stocktype, pagesize);
    let text =
        tokio::spawn(client.get(url).headers(HEADER_SSE.to_owned()).send().await?.text()).await??;
    let pos1 = text.find("total\":").ok_or(anyhow!("`Total` field not found"))? + 7;
    let pos2 = text[pos1..pos1 + 10].find("}").ok_or(anyhow!("`Total` field not found"))? + pos1;
    let n: usize = text[pos1..pos2].parse()?;
    // 注意：如果不 take 的话，split 有一半是重复的
    set.extend(text.split("COMPANY_CODE").skip(1).take(n).map(|s| format!("sh{}", &s[3..9])));
    Ok(n)
}

lazy_static::lazy_static! {
    pub static ref HEADER_SSE: reqwest::header::HeaderMap = {
        use reqwest::header::{
            HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION,
            COOKIE, PRAGMA, REFERER, USER_AGENT,
        };
        let mut header = HeaderMap::with_capacity(8);
        header.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        header.insert(PRAGMA, HeaderValue::from_static("no-cache"));
        header.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        header.insert(USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 6.1; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36"));
        header.insert(ACCEPT, HeaderValue::from_static("*/*"));
        header.insert(REFERER, HeaderValue::from_static("http://www.sse.com.cn/"));
        header.insert(ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,de;q=0.8,ko;q=0.7,ru;q=0.6,it;q=0.5,\
                ga;q=0.4,en;q=0.3"));
        header.insert(COOKIE, HeaderValue::from_static(
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
        header
    };

    // -H 'X-Request-Type: ajax' \
    // -H 'X-Requested-With: XMLHttpRequest' \
    pub static ref HEADER_SZSE: reqwest::header::HeaderMap = {
        use reqwest::header::{
            HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION,
            PRAGMA, REFERER, USER_AGENT, CONTENT_TYPE, DNT
        };
        let mut header = HeaderMap::with_capacity(8);
        header.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        header.insert(PRAGMA, HeaderValue::from_static("no-cache"));
        header.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        header.insert(USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 6.1; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36"));
        header.insert(ACCEPT, HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"));
        header.insert(REFERER, HeaderValue::from_static("http://www.szse.cn/market/trend/index.html"));
        header.insert(DNT, HeaderValue::from_static("1"));
        header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        header.insert(ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,de;q=0.8,ko;q=0.7,ru;q=0.6,it;q=0.5,\
                ga;q=0.4,en;q=0.3"));
        header
    };
    pub static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}
