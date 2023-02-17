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

pub fn connection<T: Tdx>(mut tdx: T) -> Result<()>
    where <T as Tdx>::Item: std::fmt::Debug {
    println!("send: {:?}", tdx.send());
    println!("recv: {:?}", tdx.recv_parsed(&mut Tcp::new()?)?);
    Ok(())
}

#[allow(dead_code)]
pub fn connection_mut<T: Tdx>(tdx: &mut T) -> Result<()>
    where <T as Tdx>::Item: std::fmt::Debug {
    println!("send: {:?}", tdx.send());
    let res = tdx.recv_parsed(&mut Tcp::new()?)?;
    println!("recv: {res:?}");
    Ok(())
}
