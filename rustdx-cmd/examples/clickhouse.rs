use anyhow::Result;
use odbc_api::Cursor;
use odbc_api::Environment;

fn main() -> Result<()> {
    let env = unsafe { Environment::new() }?;

    // DataSourceInfo {
    //     server_name: "ClickHouse DSN (ANSI)",
    //     driver: "ClickHouse ODBC Driver (ANSI)",
    // }
    // DataSourceInfo {
    //     server_name: "ClickHouse DSN (Unicode)",
    //     driver: "ClickHouse ODBC Driver (Unicode)",
    // }
    for data_source in env.data_sources()? {
        println!("{:#?}", data_source);
    }
    let conn = env.connect("ClickHouse DSN (Unicode)", "", "")?;
    if let Some(cursor) = conn.execute("SELECT AVG(counts) FROM tutorial.gbbq_code", ())? {
        // Use cursor to process query results.
        let headline: Vec<String> = cursor.column_names()?.collect::<Result<_, _>>()?;
        println!("{:?}", headline);
    }
    Ok(())
}
