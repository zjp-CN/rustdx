use serde::{Deserialize, Serialize};

use crate::bytes_helper::*;
use crate::file::gbbq::KEY;

// 用于（反）序列化：比如读取或写入到 csv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gbbq {
    pub sh:       u8,
    /// 6 位股票代码
    pub code:     String,
    /// `%Y-%m-%d` 格式的日期
    pub date:     String,
    /// 信息类型
    ///
    /// |   | 类别                           |    | 类别                                    |
    /// | - | ------------------------------ | -- | --------------------------------------- |
    /// | 1 | 除权除息                       | 8  | 增发新股上市                            |
    /// | 2 | 送配股上市                     | 9  | 转配股上市                              |
    /// | 3 | 非流通股上市股                 | 10 | 可转债上市                              |
    /// | 4 | 未知股本变动                   | 11 | 扩缩股 songgu:比例                      |
    /// | 5 | 股本变化                       | 12 | 非流通股缩股 songgu: 比例               |
    /// | 6 | 增发新股 peigujia; songgu:未知 | 13 | 送认购权证 songgu: 份数; hongli: 行权价 |
    /// | 7 | 股份回购                       | 14 | 送认沽权证 songgu: 份数; hongli: 行权价 |
    pub category: u8,
    /// 分红（每 10 股派现金 x 元）| 前流通盘：
    pub fh_qltp:  f32,
    /// 配股价（每股配股价 x 元）| 前总股本
    pub pgj_qzgb: f32,
    /// 送转股（每 10 股送转股比例 x 股） | 后流通盘
    pub sg_hltp:  f32,
    /// 配股（每 10 股配股比例 x 股）| 后总股本：
    pub pg_hzgb:  f32,
}

impl Gbbq {
    /// 处理 29 个字节
    ///
    /// | 位置      | 0  | 1-6    | 7      | 8-11 | 12 | 13-16 | 17-20 | 21-24 | 25-28 |
    /// | --------- | -- | ------ | ------ | ---- | -- | ----- | ----- | ----- | ----- |
    /// | py 含义   | B  | 6s     | \0x00  | I    | B  | f     | f     | f     | f     |
    /// | Rust 类型 | u8 | String | 不解析 | u32  | u8 | f32   | f32   | f32   | f32   |
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { sh:       u8_from_le_bytes(bytes, 0),
               code:     unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(1..7)) }.into(),
               date:     date_string(u32_from_le_bytes(bytes, 8)),
               category: u8_from_le_bytes(bytes, 12),
               fh_qltp:  f32_from_le_bytes(bytes, 13),
               pgj_qzgb: f32_from_le_bytes(bytes, 17),
               sg_hltp:  f32_from_le_bytes(bytes, 21),
               pg_hzgb:  f32_from_le_bytes(bytes, 25), }
    }
}

#[derive(Debug, Clone)]
pub struct Gbbqs {
    pub data: Vec<u8>,
    /// 股本变迁的记录条数
    pub len:  usize,
    pos:      usize,
}

impl Gbbqs {
    pub async fn from_file(p: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let data = std::fs::read(p)?;
        let len = u32_from_le_bytes(&data, 0) as usize;
        Ok(Self { data, len, pos: 4 })
    }
}

impl Iterator for Gbbqs {
    type Item = Gbbq;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 29 <= self.data.len() {
            self.pos += 29;
            Some(Gbbq::from_bytes(&parse(unsafe {
                                      self.data.get_unchecked(self.pos - 29..self.pos)
                                  })))
        } else {
            None
        }
    }
}

pub fn parse(encrypt: &[u8]) -> [u8; 29] {
    let mut item = [0; 29];
    let mut pos = 0usize;
    for i in (0usize..24).step_by(8) {
        let mut eax = u32_from_le_bytes(KEY, 0x44);
        let mut ebx = u32_from_le_bytes(encrypt, pos);
        let mut num = eax ^ ebx;
        let mut numold = u32_from_le_bytes(encrypt, pos + 4);
        for j in (4usize..68).step_by(4).rev() {
            ebx = (num & 0xff0000) >> 16;
            eax = u32_from_le_bytes(KEY, ebx as usize * 4 + 0x448);
            ebx = num >> 24;
            let mut eax_add = u32_from_le_bytes(KEY, ebx as usize * 4 + 0x48);
            eax = eax.overflowing_add(eax_add).0;
            ebx = (num & 0xff00) >> 8;
            let mut eax_xor = u32_from_le_bytes(KEY, ebx as usize * 4 + 0x848);
            eax ^= eax_xor;
            ebx = num & 0xff;
            eax_add = u32_from_le_bytes(KEY, ebx as usize * 4 + 0xc48);
            eax = eax.overflowing_add(eax_add).0;
            eax_xor = u32_from_le_bytes(KEY, j);
            eax ^= eax_xor;
            ebx = num;
            num = numold ^ eax;
            numold = ebx;
        }
        numold ^= u32_from_le_bytes(KEY, 0);
        unsafe { (item.get_unchecked_mut(i..i + 4)).swap_with_slice(&mut numold.to_le_bytes()) };
        unsafe { (item.get_unchecked_mut(i + 4..i + 8)).swap_with_slice(&mut num.to_le_bytes()) };
        pos += 8;
    }
    let mut end = [0u8; 5];
    end.copy_from_slice(&encrypt[24..]);
    unsafe { (item.get_unchecked_mut(24..)).swap_with_slice(&mut end) };
    item
}
