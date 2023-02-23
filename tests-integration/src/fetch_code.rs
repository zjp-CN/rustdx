#[test]
#[ignore = "联网更新数据"]
fn offical_stocks() {
    use crate::{now, shot, snap, to_table};
    use rustdx_cmd::fetch_code::{offical_stocks, StockList};

    shot!(now(), @"2023-02-23 16:28:04.861102493 +08:00");

    let mut set = StockList::with_capacity(6000);
    let (count, time) = elapse!(offical_stocks(&mut set).unwrap());
    let len = count.count();
    let set = set.into_iter().collect::<std::collections::BTreeSet<_>>();
    shot!("SHSZ", to_table(&set));
    assert_eq!(set.len(), len); // 似乎有时出现不相等

    snap!(count, @r###"
    SHSZ {
        sh1: 1646,
        sh8: 407,
        sz: 2745,
    }
    "###);
    shot!(len, @"4798");
    shot!(time, @"894");
}
