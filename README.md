# rustdx

[<img alt="github" src="https://img.shields.io/badge/MIT--LICENSE-zjp--CN%2Frustdx-blue?style=flat" height="20">](https://github.com/zjp-CN/rustdx)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rustdx?style=flat&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rustdx)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rustdx-66c2a5?style=flat&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/rustdx)
[<img alt="build status" src="https://github.com/zjp-CN/rustdx/workflows/Release%20CI/badge.svg" height="20">](https://github.com/zjp-CN/rustdx/actions)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rustdx-cmd?style=flat&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rustdx-cmd)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rustdx-cmd-66c2a5?style=flat&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/rustdx-cmd)

[![](https://img.shields.io/crates/d/rustdx.svg?label=downloads+rustdx&style=social)](https://crates.io/crates/rustdx)
[![](https://img.shields.io/crates/dv/rustdx.svg?label=downloads@latest+rustdx&style=social)](https://crates.io/crates/rustdx)
[![](https://img.shields.io/crates/d/rustdx-cmd.svg?label=downloads+rustdx-cmd&style=social)](https://crates.io/crates/rustdx-cmd)
[![](https://img.shields.io/crates/dv/rustdx-cmd.svg?label=downloads@latest+rustdx-cmd&style=social)](https://crates.io/crates/rustdx-cmd)

受 [pytdx](https://pypi.org/project/pytdx/1.28) 启发的 A 股数据获取工具，包含：
1. 一个 Rust 通用库 [rustdx](https://crates.io/crates/rustdx)；
2. 一个命令行工具 [rustdx-cmd](https://crates.io/crates/rustdx-cmd)。

命令行工具（统计数据基于笔者的单核 CPU Ubuntu 系统 release build，以实际速度为准）：
1. 解析所有最新股票列表的历史 A 股数据（包含复权数据）不到 30s ，解析后的 csv 大小 1G 多；
2. 子命令直接将解析后的 csv 数据插入到 ClickHouse （20s，表 268 M） 或 MongoDB （7 分钟，表超过 700 M）；
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
$ CARGO_PROFILE_RELEASE_LTO=true CARGO_PROFILE_RELEASE_OPT_LEVEL=3 cargo install rustdx-cmd
```

3. cargo build：
```console
$ git clone https://github.com/zjp-CN/rustdx.git
$ cd rustdx
$ cargo build -p rustdx-cmd --release
```

### 子命令

- day：解析通达信 day 文件，具体查看帮助 `rustdx day --help`、`rustdx day -h o -h l`。
- east：获取东方财富当日 A 股数据。

### 完整使用例子

准备好 day 文件、gbbq 文件和 ClickHouse 数据库：

```console
# 解析所有最新股票的历史日线数据，且计算复权数据
$ rustdx day /vdb/tmp/tdx/sh/ /vdb/tmp/tdx/sz/ -l official -g ../assets/gbbq
# 写入 ClickHouse 数据库
$ clickhouse-client --query "INSERT INTO rustdx.factor FORMAT CSVWithNames" < stocks.csv

# 有了历史日线数据之后，每个交易日收盘之后，更新当天数据
$ rustdx east -p factor.csv
# 写入 ClickHouse 数据库
$ clickhouse-client --query "INSERT INTO rustdx.factor FORMAT CSVWithNames" < eastmoney.csv
```

其中 factor.csv 来自数据库中，前一天的复权数据，ClickHouse 的导出命令：
```sql
WITH (
        SELECT max(date) AS date
        FROM rustdx.factor
    ) AS latest
SELECT
    date,
    code,
    close,
    factor
FROM rustdx.factor
WHERE latest = date
INTO OUTFILE 'factor.csv'
FORMAT CSVWithNames;
```
