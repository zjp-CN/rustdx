# rustdx

[<img alt="github" src="https://img.shields.io/github/license/zjp-CN/rustdx?color=blue" height="20">](https://github.com/zjp-CN/rustdx)
[<img alt="github" src="https://img.shields.io/github/issues/zjp-CN/rustdx?color=db2043" height="20">](https://github.com/zjp-CN/rustdx/issues)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rustdx-complete?style=flat&color=fc8d62&logo=rust&label=rustdx-complete" height="20">](https://crates.io/crates/rustdx-complete)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rustdx-66c2a5?style=flat&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/rustdx)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rustdx-cmd?style=flat&color=fc8d62&logo=rust&label=rustdx-cmd" height="20">](https://crates.io/crates/rustdx-cmd)
[<img alt="build status" src="https://github.com/zjp-CN/rustdx/workflows/Release%20CI/badge.svg" height="20">](https://github.com/zjp-CN/rustdx/actions)

[![](https://img.shields.io/crates/d/rustdx.svg?label=downloads+rustdx&style=social)](https://crates.io/crates/rustdx)
[![](https://img.shields.io/crates/dv/rustdx.svg?label=downloads@latest+rustdx&style=social)](https://crates.io/crates/rustdx)
[![](https://img.shields.io/crates/d/rustdx-cmd.svg?label=downloads+rustdx-cmd&style=social)](https://crates.io/crates/rustdx-cmd)
[![](https://img.shields.io/crates/dv/rustdx-cmd.svg?label=downloads@latest+rustdx-cmd&style=social)](https://crates.io/crates/rustdx-cmd)

受 [pytdx](https://pypi.org/project/pytdx/1.28) 启发的 A 股数据获取工具，包含：
1. 一个 Rust 通用库 [rustdx](https://crates.io/crates/rustdx)；
2. 一个命令行工具 [rustdx-cmd](https://crates.io/crates/rustdx-cmd)。

## rustdx 库使用

rustdx 是一个功能完整的 A 股数据获取库，完全对标 pytdx 的核心功能。

### 功能特性

| 功能 | rustdx 模块 | pytdx 对应 | 说明 |
|------|------------|-----------|------|
| 日K线 | `Kline` | `get_security_bars` | 支持多种周期（日/周/月/分钟） |
| 除权数据 | `Xdxr` | `get_xdxr` | 股票除权除息信息 |
| 实时行情 | `SecurityQuotes` | `get_security_quotes` | 股票和指数实时快照 |
| 股票列表 | `SecurityList` | `get_security_list` | 获取所有股票代码 |
| 分时数据 | `MinuteTime` | `get_minute_time_data` | 当日分时成交数据 |
| 逐笔成交 | `Transaction` | `get_transaction_data` | tick-level 成交数据 |
| 财务信息 | `FinanceInfo` | `get_finance_info` | 32个财务基本面数据 |
| 指数行情 | `SecurityQuotes` | `get_index_quotes` | 上证指数、深证成指等 |

### 安装

```toml
[dependencies]
rustdx-complete = "0.5.0"
```

### 使用示例

#### 获取股票实时行情

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::SecurityQuotes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tcp = Tcp::new()?;

    // 获取多只股票的实时行情
    let mut quotes = SecurityQuotes::new(vec![
        (0, "000001"),  // 平安银行（深市）
        (1, "600000"),  // 浦发银行（沪市）
    ]);

    quotes.recv_parsed(&mut tcp)?;

    for quote in quotes.result() {
        println!("{}: {} - 当前价: {}", quote.code, quote.name, quote.price);
    }

    Ok(())
}
```

#### 获取指数行情

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::SecurityQuotes;

let mut tcp = Tcp::new()?;

// 获取主要指数行情
let mut quotes = SecurityQuotes::new(vec![
    (1, "000001"),  // 上证指数
    (0, "399001"),  // 深证成指
    (1, "000300"),  // 沪深300
]);

quotes.recv_parsed(&mut tcp)?;

for quote in quotes.result() {
    println!("{}: {} (涨跌: {}%)", quote.code, quote.price, quote.change_percent);
}
```

#### 获取日线数据

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::Kline;

let mut tcp = Tcp::new()?;
let mut kline = Kline::new(1, "600000", 9, 0, 10); // 沪市、浦发银行、日线、从0开始获取10条

kline.recv_parsed(&mut tcp)?;

for bar in kline.result() {
    println!("{} : 开({}) 高({}) 低({}) 收({})",
        bar.dt, bar.open, bar.high, bar.low, bar.close);
}
```

#### 获取财务信息

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::FinanceInfo;

let mut tcp = Tcp::new()?;
let mut finance = FinanceInfo::new(0, "000001"); // 深市、平安银行

finance.recv_parsed(&mut tcp)?;

let info = &finance.result()[0];
println!("股票代码: {}", info.code);
println!("总股本: {:.0} 股", info.zongguben);
println!("净资产: {:.0} 元", info.jingzichan);
println!("净利润: {:.0} 元", info.jinglirun);
```

#### 获取分时数据

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::MinuteTime;

let mut tcp = Tcp::new()?;
let mut minute = MinuteTime::new(0, "000001", 0); // 深市、平安银行、从第0条开始

minute.recv_parsed(&mut tcp)?;

for data in minute.result().iter().take(10) { // 只打印前10条
    println!("{} : 价格={} 成交量={}", data.time, data.price, data.vol);
}
```

#### 获取逐笔成交

```rust
use rustdx::tcp::{Tcp, Tdx};
use rustdx::tcp::stock::Transaction;

let mut tcp = Tcp::new()?;
let mut transaction = Transaction::new(0, "000001", 0); // 深市、平安银行、从第0条开始

transaction.recv_parsed(&mut tcp)?;

for data in transaction.result().iter().take(5) { // 只打印前5笔
    println!("{} : 价格={} 成交量={} 买卖方向={}",
        data.time, data.price, data.vol, data.buyorsell);
}
```

### 市场代码说明

- `0` = 深市（深圳证券交易所）
- `1` = 沪市（上海证券交易所）

### 超时设置

默认 TCP 超时时间为 5 秒。如果网络环境较差，可以调整 `src/tcp/mod.rs` 中的 `TIMEOUT` 常量。

### 完整示例程序

项目 `examples/` 目录下提供了完整的使用示例：

- `test_security_quotes.rs` - 股票和指数行情
- `test_kline.rs` - K线数据
- `test_finance_info.rs` - 财务信息
- `test_minute_time.rs` - 分时数据
- `test_transaction.rs` - 逐笔成交
- `test_security_list.rs` - 股票列表

运行示例：
```bash
cargo run --example test_security_quotes
```

---

命令行工具（统计数据基于笔者的单核 CPU Ubuntu 系统 release build，以实际速度为准）：
1. 解析所有最新股票列表的历史 A 股数据（包含复权数据）不到 30s ，解析后的 csv 大小 1G 多；
2. 将解析后的 csv 数据插入到 ClickHouse （20s，表 268 M） 或 MongoDB （7 分钟，表超过 700 M）；
3. 东财日线增量更新（包括复权），2s 更新完。

关于复权：
1. 使用涨跌幅复权算法，无需修改（重算）历史复权信息；
2. 只计算收盘价前复权，其他价格复权只需基于收盘价和相对价格即可计算出来（这在 ClickHouse 中很快）。

具体文档待补充。

## rustdx-cmd

### 安装

使用以下一种方式即可：

1. 下载 [已编译的 release 版本](https://github.com/zjp-CN/rustdx/releases/latest)

2. cargo install：
```console
cargo install rustdx-cmd
```

3. cargo build：
```console
$ git clone https://github.com/zjp-CN/rustdx.git
$ cd rustdx
$ cargo build -p rustdx-cmd --release # 编译（二进制在 target/release 下）
$ cargo install --path rustdx-cmd     # 安装（二进制在全局 .cargo/bin 下）
```

### 子命令

- day：解析通达信 day 文件，具体查看帮助 `rustdx day --help`、`rustdx day -h o -h l`。
- east：获取东方财富当日 A 股数据，具体查看帮助 `rustdx east --help`。

### 完整使用例子

准备好 day 文件、gbbq 文件和 ClickHouse 数据库：

p.s. 请勿使用本项目 `assets/` 中的 gbbq 文件，因为那对你来说是过时的。

> 注意：
>
> 此工具的主要目的就是快速补齐历史日线数据，但**没有**校验交易日数据连续或者清空数据库的功能。
>
> 因没有每天记录日线导致日线不完整（或者其他原因导致数据有问题），请**重新**解析和存储所有历史数据。
>
> 重新存储数据之前，使用以下 sql 命令（以 ClickHouse 为例）删除历史数据：
>
> ```sql
> TRUNCATE TABLE rustdx.factor;
> ```
>
> 如果发现历史数据不正确，请提交 [issue](https://github.com/zjp-CN/rustdx/issues)。

```console
# 解析所有最新股票的历史日线数据，且计算复权数据
$ rustdx day /vdb/tmp/tdx/sh/ /vdb/tmp/tdx/sz/ -l official -g ../assets/gbbq -t rustdx.factor
# 写入 ClickHouse 数据库
$ clickhouse-client --query "INSERT INTO rustdx.factor FORMAT CSVWithNames" < stocks.csv

# 有了历史日线数据之后，每个交易日收盘之后，更新当天数据
$ rustdx east -p factor.csv -t rustdx.factor
# 写入 ClickHouse 数据库
$ clickhouse-client --query "INSERT INTO rustdx.factor FORMAT CSVWithNames" < eastmoney.csv
```

其中 factor.csv 来自数据库中，前一天的复权数据，ClickHouse 的导出命令：
```sql
SELECT
    yesterday() AS date,
    code,
    last_value(close) AS close,
    last_value(factor) AS factor
FROM rustdx.factor
GROUP BY code
INTO OUTFILE 'factor.csv'
FORMAT CSVWithNames;
```

---

或者：
```console
# 解析所有最新股票的历史日线数据，且计算复权数据，写入 ClickHouse 数据库
$ rustdx day /vdb/tmp/tdx/sh/ /vdb/tmp/tdx/sz/ -l official -g ../assets/gbbq -o clickhouse -t rustdx.factor

# 有了历史日线数据之后，每个交易日收盘之后，更新当天数据
$ rustdx east -p clickhouse -o clickhouse -t rustdx.factor
```

## CHANGELOG

[更新记录](https://github.com/zjp-CN/rustdx/blob/main/CHANGELOG.md)

## 使用示例

### 计算任何周期的涨跌幅

```sql
SELECT
    code,
    toYYYYMM(date) AS m, -- 这里以月周期为例
    ((LAST_VALUE(factor) / FIRST_VALUE(factor)) * FIRST_VALUE(close)) / FIRST_VALUE(preclose) AS mgrowth
FROM rustdx.factor -- 命令行参数中所写入的表名，假设你按照我上面给的命令行示例运行，那么原始数据在这个表
GROUP BY code, m   -- 按照月聚合
ORDER BY code ASC, m DESC;
```

为什么 `mgrowth` 是那样计算，见 [涨跌幅复权与前复权](https://zjp-cn.github.io/posts/qfq/)。

### 计算前复权价格

注意，上面计算涨幅时没有计算前复权价格，但大部分情况下必须知道前复权价格来计算价格相关的指标。

那么可以每日数据成功入库之后，运行一次以下脚本，注意：
* 这基于最新价来计算所有股票的所有历史前复权价格（在我的单核机器上需要 11 秒）
* 每次运行脚本会把之前的计算结果清空
* 前复权的结果在 `rustdx.qfq` 这个表（只有股票代码和价格）

```sql
-- 计算前复权价格
DROP TABLE IF EXISTS rustdx.qfq_x; -- 临时表
CREATE TABLE rustdx.qfq_x (
    code FixedString(6),
    x    Float64,
    PRIMARY KEY(code)
) ENGINE = MergeTree  AS 
WITH
qfq AS (
    SELECT code, LAST_VALUE(close) / LAST_VALUE(factor) AS qfq_multi
    FROM rustdx.factor
    GROUP BY code
    ORDER BY code
)
SELECT * FROM qfq;

DROP TABLE IF EXISTS rustdx.qfq; -- 前复权价格
CREATE TABLE rustdx.qfq (
    date  Date,
    code  FixedString(6),
    close Float64,
    open  Float64,
    high  Float64,
    low   Float64,
    PRIMARY KEY(date, code)
) ENGINE = MergeTree AS
WITH
qfq_x AS (SELECT * FROM rustdx.qfq_x),
fct AS (
    SELECT date, code, open/close AS open, high/close AS high, low/close AS low, factor
    FROM rustdx.factor
),
raw AS (
    SELECT *
    FROM fct
    LEFT JOIN qfq_x ON qfq_x.code = fct.code
)
SELECT date, code, factor*x AS close, open*close AS open, high*close AS high, low*close AS low
FROM raw
ORDER BY date, code
```
