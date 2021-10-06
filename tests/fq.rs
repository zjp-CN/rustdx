use insta::assert_debug_snapshot;

#[test]
fn day_sz000001() -> rustdx::Result<()> {
    use rustdx::file::{
        day::Day,
        gbbq::{Fq, Gbbq},
    };
    let day_src = std::fs::read("tests/assets/sz000001.day")?;
    let days = day_src.chunks_exact(32).map(|arr| Day::from_bytes(1, arr));

    let mut gbbq_src = std::fs::read("tests/assets/gbbq")?;
    let stock_gbbq = Gbbq::filter_hashmap(Gbbq::iter(&mut gbbq_src[4..]));

    let fq = Fq::new(days, stock_gbbq.get(&1).unwrap()).unwrap();
    assert_debug_snapshot!(&fq[..3]);
    Ok(())
}
