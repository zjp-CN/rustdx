// 测试东财股票日线数据
#[test]
#[ignore = "联网更新数据"]
fn daily() {
    use super::{now, shot, to_table, Tabled};
    use rustdx_cmd::eastmoney::{fetch, Day};

    // 此测试运行的日期
    shot!(now(), @"2023-02-23 15:38:52.329844174 +08:00");

    let (data, elapse_get) = elapse!(fetch(None).unwrap());
    shot!(elapse_get, @"358"); // 获取数据的耗时

    let iter = data.data.diff.into_iter().filter_map(Data::try_from);
    shot!("东财-股票-json", to_table(iter));

    #[derive(Tabled)]
    pub struct Data {
        pub code: String,
        pub open: f32,
        pub high: f32,
        pub low: f32,
        pub close: f32,
        pub amount: f32,
        pub vol: f32,
        pub preclose: f32,
    }
    impl Data {
        fn try_from(d: Day) -> Option<Data> {
            Some(Data {
                code: d.code,
                open: d.open?,
                high: d.high?,
                low: d.low?,
                close: d.close?,
                amount: d.amount?,
                vol: d.vol?,
                preclose: d.preclose?,
            })
        }
    }
}
