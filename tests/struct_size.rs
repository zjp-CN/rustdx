use insta::assert_debug_snapshot;
use std::mem::size_of;

#[test]
fn tcp() {
    use rustdx::tcp;
    assert_debug_snapshot!(size_of::<tcp::Tcp>(),              @"72");
    assert_debug_snapshot!(size_of::<tcp::SecurityCount>(),    @"24");
    assert_debug_snapshot!(size_of::<tcp::SecurityList>(),     @"72");
    assert_debug_snapshot!(size_of::<tcp::SecurityListData>(), @"48");
    assert_debug_snapshot!(size_of::<tcp::stock::Kline>(),     @"88");
    assert_debug_snapshot!(size_of::<tcp::stock::KlineData>(), @"80");
    assert_debug_snapshot!(size_of::<tcp::stock::Xdxr>(),      @"104");
    assert_debug_snapshot!(size_of::<tcp::stock::XdxrData>(),  @"48");
}
