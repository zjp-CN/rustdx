use crate::{
    file::gbbq::{Factor, Fq, Gbbq},
    Error::Custom,
    Result,
};
use std::path::Path;

/// 注意：成交量的单位为 “手”，而不是股。在通达信和交易所数据里，单位为股。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Day {
    pub date:     String,
    pub code:     String,
    pub open:     f32,
    pub high:     f32,
    pub low:      f32,
    pub close:    f32,
    pub amount:   f32,
    pub vol:      f32,
    pub preclose: f64,
    pub factor:   f64,
}

impl Day {
    pub fn new(code: u32, p: impl AsRef<Path>, gbbqs: Option<&[Gbbq]>) -> Result<Vec<Self>> {
        let raw = std::fs::read(p)?;
        let days = raw.chunks_exact(32).map(|b| super::Day::from_bytes(code, b));
        let fq = gbbqs.map(|g| Fq::new(days.clone(), g))
                      .unwrap_or(Fq::no_gbbq(days.clone()))
                      .ok_or(Custom("复权失败"))?
                      .into_iter()
                      .filter(|d| d.trading);
        Ok(days.zip(fq)
               .map(|(d, f)| Self { date:     crate::bytes_helper::date_string(d.date),
                                    code:     format!("{:06}", d.code),
                                    open:     d.open,
                                    high:     d.high,
                                    low:      d.low,
                                    close:    d.close,
                                    amount:   d.amount,
                                    vol:      d.vol as f32 / 100.,
                                    preclose: f.preclose,
                                    factor:   f.factor, })
               .collect())
    }

    pub fn concat(code: u32, p: impl AsRef<Path>, gbbqs: Option<&[Gbbq]>, f: Option<&Factor>)
                  -> Result<Vec<Self>> {
        let raw = std::fs::read(p)?;
        let days = raw.chunks_exact(32).map(|b| super::Day::from_bytes(code, b));
        let (preclose, factor) = f.map(|f| (f.preclose, f.factor))
                                  .unwrap_or((days.clone()
                                                  .next()
                                                  .ok_or(Custom("初始化前收盘价失败"))?
                                                  .close as f64,
                                              1.));
        let fq = gbbqs.map(|g| Fq::concat(days.clone(), g, preclose, factor))
                      .unwrap_or(Fq::no_gbbq(days.clone()))
                      .ok_or(Custom("复权失败"))?
                      .into_iter()
                      .filter(|d| d.trading);
        Ok(days.zip(fq)
               .map(|(d, f)| Self { date:     crate::bytes_helper::date_string(d.date),
                                    code:     format!("{:06}", d.code),
                                    open:     d.open,
                                    high:     d.high,
                                    low:      d.low,
                                    close:    d.close,
                                    amount:   d.amount,
                                    vol:      d.vol as f32 / 100.,
                                    preclose: f.preclose,
                                    factor:   f.factor, })
               .collect())
    }
}
