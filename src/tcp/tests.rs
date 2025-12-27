/// 多个值一起比较格式化的字符串值，当遇到第一对不相等的值比较时，直接 panic 。
/// 这主要用于比较不实现 PartialEq trait 类型的值，且只用于内部测试代码。
macro_rules! compare {
    ($a:expr, $($b:expr),+) => {
       {
            let v = format!("{:?}", $a);
            $(
                assert_eq!(v, format!("{:?}", $b), "{} ≠ {}", stringify!($a), stringify!($b));
            )+
        }
    };
}

use super::{Result, Tcp, Tdx};

/// 连接测试辅助函数。
///
/// 注意：这是一个集成测试，需要实际的网络连接。
/// 如果设置了环境变量 `RUSTDX_SKIP_INTEGRATION_TESTS=1`，则会跳过测试。
pub fn connection<T: Tdx>(mut tdx: T) -> Result<()>
where
    <T as Tdx>::Item: std::fmt::Debug,
{
    if std::env::var("RUSTDX_SKIP_INTEGRATION_TESTS").is_ok() {
        println!("⚠️  跳过集成测试 (RUSTDX_SKIP_INTEGRATION_TESTS 已设置)");
        return Ok(());
    }

    println!("send: {:?}", tdx.send());
    println!("recv: {:?}", tdx.recv_parsed(&mut Tcp::new()?)?);
    Ok(())
}

#[allow(dead_code)]
pub fn connection_mut<T: Tdx>(tdx: &mut T) -> Result<()>
where
    <T as Tdx>::Item: std::fmt::Debug,
{
    if std::env::var("RUSTDX_SKIP_INTEGRATION_TESTS").is_ok() {
        println!("⚠️  跳过集成测试 (RUSTDX_SKIP_INTEGRATION_TESTS 已设置)");
        return Ok(());
    }

    println!("send: {:?}", tdx.send());
    let res = tdx.recv_parsed(&mut Tcp::new()?)?;
    println!("recv: {res:?}");
    Ok(())
}
