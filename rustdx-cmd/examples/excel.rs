use calamine::{open_workbook_auto, Data, Reader, Sheets};
use eyre::Result;
use std::time::Instant;

/// 貌似对于中大文件，xlsx 解析更快，对于很小的文件，xls 更快。一般使用 xlsx 即可。
fn main() -> Result<()> {
    use Exchange::*;
    read_excel("../assets/xlsx/A股列表-szse.xlsx", Szse)?;
    read_excel("../assets/xlsx/A股列表-szse.xls", Szse)?;
    // 注意：上海交易所下载的 xls 文件格式不规范，暂时无法解析，需要 Excel 另存为 xls 或 xlsx
    // read_excel("../assets/xlsx/主板A股-rawbroken.xls", "主板A股")?/* CfbError::Ole */;
    read_excel("../assets/xlsx/主板A股-sse.xlsx", Sse)?;
    read_excel("../assets/xlsx/主板A股-sse.xls", Sse)?;

    get_xlsx(Szse)?;
    read_excel("../assets/xlsx/szse.xlsx", Szse)?;

    Ok(())
}

/// Sse 股票列表数据暂时无法直接获取到，而且获取到的 xls 签名不是 ole ，无法解析。
/// 所以只能手动下载，用 excel 保存为 xlsx 或 xls 。
fn get_xlsx(ex: Exchange) -> Result<()> {
    let (url, fname) = match ex{
        Exchange::Szse=>("http://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random=0.8587844061443386","../assets/xlsx/szse.xlsx"),
        Exchange::Sse => ("http://query.sse.com.cn/security/stock/downloadStockListFile.do?csrcCode=&stockCode=&areaName=&stockType=1", "../assets/xlsx/sse.xls")
    };
    let buf = &mut Vec::with_capacity(1 << 20);
    ureq::get(url).call()?.into_reader().read_to_end(buf)?;
    std::fs::write(fname, buf)?;
    Ok(())
}

/// 交易所
enum Exchange {
    /// 上海证券交易所
    Sse,
    /// 深圳证券交易所
    Szse,
}

type FsReader = std::io::BufReader<std::fs::File>;
fn read_excel(path: &str, ex: Exchange) -> Result<Sheets<FsReader>> {
    let now = Instant::now();
    let mut workbook = open_workbook_auto(path)?;
    println!(
        "{:30}：{} s",
        path,
        now.elapsed().as_millis() as f64 / 1000.0
    );

    let pos = match_ex(ex);

    // Read whole worksheet data and provide some statistics
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let total_cells = range.get_size().0 * range.get_size().1;
        let non_empty_cells: usize = range.used_cells().count();
        println!(
            "{:?} Found {} cells in {:?}, including {} non empty cells",
            path,
            total_cells,
            workbook.sheet_names(),
            non_empty_cells
        );
        // alternatively, we can manually filter rows
        // assert_eq!(non_empty_cells,
        //            range.rows().flat_map(|r| r.iter().filter(|&c| c !=
        // &Data::Empty)).count());
        range
            .rows()
            .next_back()
            .map(|r| println!("{:?}", get_string(&r[pos])))
            .unwrap();
    }

    Ok(workbook)
}

/// 匹配交易所官网下载的 excel 文件中，股票代码在每行的位置
const fn match_ex(ex: Exchange) -> usize {
    match ex {
        Exchange::Sse => 0,
        Exchange::Szse => 4,
    }
}

/// 每个单元格被解析的类型可能会不一样，所以把股票代码统一转化成字符型
fn get_string(cell: &Data) -> std::borrow::Cow<'_, str> {
    match cell {
        Data::Int(x) => x.to_string().into(),
        Data::Float(x) => (*x as i64).to_string().into(),
        Data::String(x) => x.into(),
        _ => unreachable!(),
    }
}
