//! 此库用于测试 rustdx 和 rustdx-cmd

/// 测量一个表达式花费的时间，返回 `(val, time_in_millis)`
#[allow(unused)]
macro_rules! elapse {
    ($e:expr) => {{
        let now = std::time::Instant::now();
        let val = $e;
        let time = now.elapsed().as_millis();
        (val, time)
    }};
}

mod east;
mod fetch_code;

pub use insta::{assert_debug_snapshot as snap, assert_snapshot as shot};
use tabled::{settings::Style, Table, Tabled};

pub type DateTime = chrono::DateTime<chrono::Local>;
pub fn now() -> DateTime {
    std::time::SystemTime::now().into()
}

/// 转成表格来打印
pub fn to_table<T: Tabled, I: IntoIterator<Item = T>>(into_iter: I) -> String {
    Table::new(into_iter).with(Style::psql()).to_string()
}

/// 转成表格来打印，并自定义表头
pub fn to_table_with_headers<T, I>(into_iter: I, headers: &[&str]) -> String
where
    T: Tabled,
    I: IntoIterator<Item = T>,
{
    let mut builder = Table::builder(into_iter);
    builder.push_column(headers.iter().copied());
    builder.build().with(Style::psql()).to_string()
}
