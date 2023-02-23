#[test]
fn offical_stocks() {
    use crate::{now, shot, snap};
    use rustdx_cmd::fetch_code::{offical_stocks, StockList};

    shot!(now(), @"2023-02-23 15:50:16.053645411 +08:00");

    let mut set = StockList::with_capacity(6000);
    let (count, time) = elapse!(offical_stocks(&mut set).unwrap());
    let len = count.count();
    assert_eq!(set.len(), len);

    snap!(count, @r###"
    SHSZ {
        sh1: 1646,
        sh8: 407,
        sz: 2745,
    }
    "###);
    shot!(len, @"4798");
    shot!(time, @"795");
}
