//! 此库用于测试 rustdx 和 rustdx-cmd

mod east;

use tabled::{Table, Tabled};

pub type DateTime = chrono::DateTime<chrono::Local>;
pub fn now() -> DateTime {
    std::time::SystemTime::now().into()
}

/// 转成表格来打印
pub fn to_table<T: Tabled, I: IntoIterator<Item = T>>(into_iter: I) -> String {
    Table::new(into_iter)
        .with(tabled::Style::psql())
        .to_string()
}

/// 转成表格来打印，并自定义表头
pub fn to_table_with_headers<T, I>(into_iter: I, headers: &[&str]) -> String
where
    T: Tabled,
    I: IntoIterator<Item = T>,
{
    let mut builder = Table::builder(into_iter);
    builder.set_columns(headers.iter().copied());
    builder.build().with(tabled::Style::psql()).to_string()
}
