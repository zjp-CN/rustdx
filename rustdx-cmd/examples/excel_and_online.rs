#![feature(once_cell)]
use rustdx_cmd::fetch_code;
use std::sync::LazyLock;

macro_rules! get {
    (sz) => {{
        let mut set = ::std::collections::HashSet::with_capacity(3000);
        fetch_code::get_sz_stocks(&mut set).unwrap();
        let mut v = set.into_iter().collect::<Vec<_>>();
        v.sort();
        v
    }};
    (sh, $a:literal, $b:literal) => {{
        let mut set = ::std::collections::HashSet::with_capacity(3000);
        fetch_code::get_sh_stocks(&mut set, $a, $b).unwrap();
        let mut v = set.into_iter().collect::<Vec<_>>();
        v.sort();
        v
    }};
}

static SH8: LazyLock<Vec<String>> = LazyLock::new(|| get!(sh, "8", "1000"));
static SH1: LazyLock<Vec<String>> = LazyLock::new(|| get!(sh, "1", "2500"));
static SZ: LazyLock<Vec<String>> = LazyLock::new(|| get!(sz));

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
fn main() {
    let (sh8, sh1, sz) = (&*SH8, &*SH1, &*SZ);
    println!(
        "sh8: {}\n{:?}\n{:?}",
        sh8.len(),
        &sh8[..10],
        &sh8[sh8.len() - 10..]
    );
    println!(
        "sh1: {}\n{:?}\n{:?}",
        sh1.len(),
        &sh1[..10],
        &sh1[sh1.len() - 10..]
    );
    println!(
        "sz: {}\n{:?}\n{:?}",
        sz.len(),
        &sz[..10],
        &sz[sz.len() - 10..]
    );
}

#[test]
fn save() -> eyre::Result<()> {
    use insta::assert_debug_snapshot as snap;
    use rustdx_cmd::eastmoney;
    use std::collections::HashSet;

    let (sh8, sh1, sz) = (&*SH8, &*SH1, &*SZ);
    let (lsh8, lsh1, lsz) = (sh8.len(), sh1.len(), sz.len());
    snap!("sh1", sh1);
    snap!(lsh1, @"1679");
    snap!("sh8", sh8);
    snap!(lsh8, @"510");
    snap!("sz", sz);
    snap!(lsz, @"2758");
    let l = lsh1 + lsh8 + lsz;
    snap!(l, @"4947");

    let s = eastmoney::get(6000)?;
    let res = eastmoney::parse(&s)?;
    let east: HashSet<_> = res
        .data
        .diff
        .into_iter()
        .filter(|v| matches!(v.open, eastmoney::F32::Yes(_)))
        .map(|v| v.code)
        .collect();
    let mut v = east.iter().collect::<Vec<_>>();
    v.sort();
    let lv = v.len();
    let total = res.data.total as usize;
    eyre::ensure!(lv == total, "lv = {lv} 与 total = {total} 不相等");
    snap!("eastmoney", v);
    snap!(lv, @"5168");
    snap!(lv == l, @"false");

    let exchange = HashSet::from_iter(
        [sh8.iter().cloned(), sh1.iter().cloned(), sz.iter().cloned()]
            .into_iter()
            .flatten()
            .map(|s| s[2..].to_string()),
    );
    snap!("diff_exchange-east", &exchange - &east);
    snap!("diff_east-exchange", &east - &exchange);

    Ok(())
}
