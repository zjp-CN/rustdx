/// 把 slice 转换为长度为 4 的 array 。
/// [泛型写法](https://stackoverflow.com/a/37682288/15448980)
#[inline]
pub fn into_arr4(slice: &[u8], pos: usize) -> [u8; 4] {
    let mut arr = [0; 4];
    arr.copy_from_slice(unsafe { slice.get_unchecked(pos..pos + 4) });
    arr
}

#[inline]
pub fn u32_from_le_bytes(slice: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes(into_arr4(slice, pos))
}

#[inline]
pub fn f32_from_le_bytes(slice: &[u8], pos: usize) -> f32 {
    f32::from_le_bytes(into_arr4(slice, pos))
}

/// 把 slice 转换为长度为 2 的 array
#[inline]
pub fn into_arr2(slice: &[u8], pos: usize) -> [u8; 2] {
    let mut arr = [0; 2];
    arr.copy_from_slice(unsafe { slice.get_unchecked(pos..pos + 2) });
    arr
}

#[inline]
pub fn u16_from_le_bytes(slice: &[u8], pos: usize) -> u16 {
    u16::from_le_bytes(into_arr2(slice, pos))
}

#[inline]
pub fn u8_from_le_bytes(slice: &[u8], pos: usize) -> u8 {
    u8::from_le_bytes({
        let mut arr = [0];
        arr.copy_from_slice(unsafe { slice.get_unchecked(pos..pos + 1) });
        arr
    })
}

/// 把 6 位 u32 日期转化成 `%Y-%m-%d` 格式，比如 `20210801` => `2021-08-01`
#[inline]
pub fn date_string(x: u32) -> String {
    let [y, m, d] = [x / 10000, x % 10000 / 100, x % 10000 % 100];
    let fill = |x: u32| if x > 9 { "" } else { "0" };
    format!("{}-{}{}-{}{}", y, fill(m), m, fill(d), d)
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub fn ser_date_string<S>(date: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
    serializer.serialize_str(&crate::bytes_helper::date_string(*date))
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub fn ser_code_string<S>(code: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
    serializer.serialize_str(&format!("{:06}", code))
}
