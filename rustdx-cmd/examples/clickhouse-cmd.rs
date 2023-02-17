//! cargo run --example tmp --features tokio/macros,tokio/sync,rustdx,csv

use rustdx::serde_type::Day;
use std::{
    io::{self, Result, Write},
    process::{Command, Output, Stdio},
    time::Instant,
};

#[tokio::main]
async fn main() -> Result<()> {
    let now = Instant::now();
    db_setup()?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let manager = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            insert_into_db(data)?
        }
        Ok::<(), std::io::Error>(())
    });

    let task1 = tokio::spawn(task(
        tx.clone(),
        600198,
        "/vdb/rustdx/assets/dbg/sh600198.day",
    ));
    let task2 = tokio::spawn(task(tx, 1, "/vdb/rustdx/assets/sz000001.day"));
    // 这些任务是并行的
    task1.await.unwrap()?;
    task2.await.unwrap()?;
    manager.await.unwrap()?;
    println!("reached end of main: takes {}ms", now.elapsed().as_millis());
    Ok(())
}

const DATABASE: &'static str = "rustdx";
const TABLE: &'static str = "rustdx.day_test2";
const QUERY: &'static str = "INSERT INTO rustdx.day_test2 FORMAT CSVWithNames";

async fn task(
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    code: u32,
    path: impl AsRef<std::path::Path>,
) -> Result<usize> {
    let now = Instant::now();

    let days = Day::from_file_into_vec(code.clone(), path).unwrap();
    let len = days.len();
    let size = std::mem::size_of_val(&days[0]) * len;
    println!(
        "#{:06}# `days` size: {}K len: {} `day` size: {}",
        code,
        size as f64 / 1024.,
        len,
        std::mem::size_of::<Day>()
    );
    let data = Vec::with_capacity(size);
    // 利用 WriterBuilder + capacity 改进
    let mut wtr = csv::Writer::from_writer(data);
    for day in days {
        wtr.serialize(day).unwrap();
    }
    let data = wtr.into_inner().unwrap();
    // println!("{:?}", String::from_utf8(data.clone()).unwrap());

    tx.send(data).await.unwrap();
    println!("read data from file: takes {}ms", now.elapsed().as_millis());
    Ok(len)
}

// 清除数据，只留下表结构：TRUNCATE TABLE tutorial.gbbq_code
// 完全删除表：DROP TABLE tutorial.gbbq_code
// 统计条数：SELECT COUNT(code) FROM tutorial.gbbq_code
fn insert_into_db(data: Vec<u8>) -> Result<()> {
    let now = Instant::now();
    let mut process = Command::new("clickhouse-client")
        .args(["--query", QUERY])
        .stdin(Stdio::piped())
        // .stdout(Stdio::piped())
        // .stdout(Stdio::null())
        .spawn()?;
    // csv 序列化的 [u8] 空间比 [Day] 小 25% ，因为结构体 Day 的内存空间会对齐
    println!("`days` csv bytes len: {}", data.len());
    process.stdin.as_mut().unwrap().write_all(&data)?;
    // let output = process.wait_with_output()?;
    // check_output(output);
    println!(
        "clickhouse-client cmd: takes {}ms",
        now.elapsed().as_millis()
    );
    Ok(())
}

fn check_output(output: Output) {
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}

fn db_setup() -> Result<()> {
    let now = Instant::now();
    let create_database = format!("CREATE DATABASE IF NOT EXISTS {}", DATABASE);
    let output = Command::new("clickhouse-client")
        .args(["--query", &create_database])
        .output()?;
    check_output(output);
    #[rustfmt::skip]
    let create_table = format!("
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
        PARTITION BY toYYYYMM(date)
    ", TABLE); // PARTITION BY 部分可能需要去掉
    let output = Command::new("clickhouse-client")
        .args(["--query", &create_table])
        .output()?;
    check_output(output);
    // println!("{}", create_table);
    println!("clickhouse sets up: takes {}ms", now.elapsed().as_millis());
    Ok(())
}
