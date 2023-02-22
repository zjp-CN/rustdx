// 测试东财股票日线数据
#[test]
#[ignore = "联网更新数据"]
fn daily() {
    use super::{now, to_table, Tabled};
    use insta::assert_display_snapshot as snap;
    use rustdx_cmd::eastmoney::{get, parse, Day, F32};
    use std::fmt::Display;
    use std::time::Instant;

    // 此测试运行的日期
    snap!(now(), @"2023-02-22 23:18:32.634160957 +08:00");

    let now = Instant::now();
    let text = get(6000).unwrap();
    let elapse_get = now.elapsed().as_millis();
    snap!(elapse_get, @"993"); // 获取数据的耗时

    snap!("东财-股票-文本", &text);

    let data = parse(&text).unwrap();
    let iter = data.data.diff.into_iter().filter_map(Data::try_from);
    snap!("东财-股票-json", to_table(iter));

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
