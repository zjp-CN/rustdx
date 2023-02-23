// 测试东财股票日线数据
#[test]
#[ignore = "联网更新数据"]
fn daily() {
    use super::{now, shot, to_table, Tabled};
    use rustdx_cmd::eastmoney::{get, parse, Day, F32};
    use std::fmt::Display;

    // 此测试运行的日期
    shot!(now(), @"2023-02-23 15:38:52.329844174 +08:00");

    let (text, elapse_get) = elapse!(get(6000).unwrap());
    shot!(elapse_get, @"358"); // 获取数据的耗时

    shot!("东财-股票-文本", &text);

    let data = parse(&text).unwrap();
    let iter = data.data.diff.into_iter().filter_map(Data::try_from);
    shot!("东财-股票-json", to_table(iter));

    pub struct Float(f32);
    impl Display for Float {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    impl Float {
        fn try_from(f: F32<'_>) -> Option<Float> {
            match f {
                F32::Null(_) => None,
                F32::Yes(f) => Some(Float(f)),
            }
        }
    }

    #[derive(Tabled)]
    pub struct Data {
        pub code: String,
        pub open: Float,
        pub high: Float,
        pub low: Float,
        pub close: Float,
        pub amount: Float,
        pub vol: Float,
        pub preclose: Float,
    }
    impl Data {
        fn try_from(d: Day<'_>) -> Option<Data> {
            Some(Data {
                code: d.code,
                open: Float::try_from(d.open)?,
                high: Float::try_from(d.high)?,
                low: Float::try_from(d.low)?,
                close: Float::try_from(d.close)?,
                amount: Float::try_from(d.amount)?,
                vol: Float::try_from(d.vol)?,
                preclose: Float::try_from(d.preclose)?,
            })
        }
    }
}
