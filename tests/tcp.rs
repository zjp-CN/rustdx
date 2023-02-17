// use insta::{assert_debug_snapshot, assert_yaml_snapshot};
use insta::assert_debug_snapshot;
use rustdx::tcp::{self, Tcp, Tdx};
use std::io::Result;

#[test]
fn tcp_security_count() -> Result<()> {
    let mut tcp = Tcp::new()?;

    let mut count = tcp::SecurityCount::new(0); // sz
    let c = *count.recv_parsed(&mut tcp)?;
    assert_debug_snapshot!("security-count-sz", count);
    assert_debug_snapshot!(c, @"13471");

    let mut count = tcp::SecurityCount::new(1); // sh
    let c = *count.recv_parsed(&mut tcp)?;
    assert_debug_snapshot!("security-count-sh", count);
    assert_debug_snapshot!(c, @"18065");

    Ok(())
}

#[test]
fn tcp_security_list() -> Result<()> {
    let mut list = tcp::SecurityList::default(); // sz
    assert_debug_snapshot!("security-list-send", list.send);
    list.recv_parsed(&mut Tcp::new()?)?;
    assert_debug_snapshot!("security-list-count", list.count);
    // assert_yaml_snapshot!("security-list-recv", list.data);
    Ok(())
}
