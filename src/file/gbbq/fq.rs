use super::Gbbq;
use crate::file::day::Day;

#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Factor {
    pub date: String,
    pub code: String,
    #[serde(rename(deserialize = "close"))]
    pub preclose: f64,
    pub factor: f64,
}

#[cfg(feature = "serde")]
impl Factor {
    /// 根据前收、收盘价（最新价）和前一日的因子计算当日的复权因子。
    #[inline]
    pub fn compute_factor(&self, close: f64) -> f64 {
        self.factor * (close / self.preclose)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fq {
    pub code: u32,
    /// 年月日
    pub date: u32,
    pub factor: f64,
    pub close: f64,
    pub preclose: f64,
    pub trading: bool,
    pub xdxr: bool,
}

impl Fq {
    /// 从上市日开始计算复权。
    ///
    /// ## 注意
    /// 1. 上市日因子为 1。所以如果存在除权日先于上市日，直接舍弃先于上市日的除权日，
    ///    比如 #000001#、#601975#。
    /// 2. 解析后的交易日天数、除权日天数会在 debug build 下校验。
    pub fn new(
        days: impl Iterator<Item = Day> + ExactSizeIterator + Clone, g1: &[Gbbq],
    ) -> Option<Vec<Fq>> {
        let count = days.len();
        let mut fqs: Vec<Fq> = Vec::with_capacity(count + 128);
        let mut preclose = days.clone().next()?.close as f64;
        let mut factor = 1.;
        let mut gbbq = g1.iter();
        let mut xdxr = gbbq.next()?;
        let mut last = false;
        let mut first = 0usize;

        for (i, d) in days.enumerate() {
            while !last && d.date > xdxr.date {
                // 因为停牌或某种原因导致下个交易日晚于除权日
                if i == 0 {
                    // 为了让上市日因子为 1
                    first += 1;
                } else {
                    fqs.push(Self::_0(d, xdxr, preclose, &mut factor, false, true));
                }
                if let Some(x) = gbbq.next() {
                    xdxr = x;
                } else if !last {
                    last = true;
                }
            }
            if d.date == xdxr.date {
                // 除权日且交易日
                fqs.push(Self::_0(d, xdxr, preclose, &mut factor, true, true));
                if let Some(x) = gbbq.next() {
                    xdxr = x;
                } else if !last {
                    last = true;
                }
            } else if d.date < xdxr.date || last {
                // 下个除权日之前的交易日，或者最后一个除权日的交易日
                fqs.push(Self::_0(d, xdxr, preclose, &mut factor, true, false));
            }
            preclose = d.close as f64;
        }

        // 确保所有数据都被正确解析：必须满足两个条件
        debug_assert_eq!(
            count,
            fqs.iter().filter(|f| f.trading).count(),
            "{} 交易日天数不正确",
            g1[0].code
        );
        debug_assert_eq!(
            if g1.last().unwrap().date <= fqs.last().unwrap().date {
                g1.len()
            } else {
                // gbbq 最新日期大于 day 文件最新日期
                g1.iter().take_while(|g| g.date <= fqs.last().unwrap().date).count()
            },
            fqs.iter().filter(|f| f.xdxr).count() + first,
            "{} 除权除息日天数不正确：\nleft: {:?}\nright: {:?}",
            g1[0].code,
            g1,
            fqs.iter().filter(|f| f.xdxr).collect::<Vec<_>>()
        );

        Some(fqs)
    }

    pub fn concat(
        days: impl Iterator<Item = Day> + ExactSizeIterator + Clone, g1: &[Gbbq],
        mut preclose: f64, mut factor: f64,
    ) -> Option<Vec<Fq>> {
        let count = days.len();
        let mut fqs: Vec<Fq> = Vec::with_capacity(count + 128);
        let mut gbbq = g1.iter();
        let mut xdxr = gbbq.next()?;
        let mut last = false;

        for d in days {
            while !last && d.date > xdxr.date {
                // 因为停牌或某种原因导致下个交易日晚于除权日
                // fqs.push(Self::_0(d, xdxr, preclose, &mut factor, false, true));
                if let Some(x) = gbbq.next() {
                    xdxr = x;
                } else if !last {
                    last = true;
                }
            }
            if d.date == xdxr.date {
                // 除权日且交易日
                fqs.push(Self::_0(d, xdxr, preclose, &mut factor, true, true));
                if let Some(x) = gbbq.next() {
                    xdxr = x;
                } else if !last {
                    last = true;
                }
            } else if d.date < xdxr.date || last {
                // 下个除权日之前的交易日，或者最后一个除权日的交易日
                fqs.push(Self::_0(d, xdxr, preclose, &mut factor, true, false));
            }
            preclose = d.close as f64;
        }

        // 确保所有数据都被正确解析：必须满足两个条件
        debug_assert_eq!(
            count,
            fqs.iter().filter(|f| f.trading).count(),
            "{} 交易日天数不正确",
            g1[0].code
        );
        debug_assert_eq!(
            g1.iter()
                .filter(
                    |g| g.date >= fqs.first().unwrap().date && g.date <= fqs.last().unwrap().date
                )
                .count(),
            fqs.iter().filter(|f| f.xdxr).count(),
            "{} 除权除息日天数不正确：\nleft: {:?}\nright: {:?}",
            g1[0].code,
            g1,
            fqs.iter().filter(|f| f.xdxr).collect::<Vec<_>>()
        );

        Some(fqs)
    }

    pub fn no_gbbq(days: impl Iterator<Item = Day> + ExactSizeIterator + Clone) -> Option<Vec<Fq>> {
        let mut preclose = days.clone().next()?.close as f64;
        let mut factor = 1.;
        Some(
            days.map(|d| {
                let close = d.close as f64;
                factor *= close / preclose;
                let fq = Self {
                    close,
                    factor,
                    preclose,
                    trading: true,
                    xdxr: false,
                    code: d.code,
                    date: d.date,
                };
                preclose = close;
                fq
            })
            .collect(),
        )
    }

    #[inline]
    fn _0(d: Day, g: &Gbbq, preclose: f64, factor: &mut f64, trading: bool, xdxr: bool) -> Self {
        let [preclose, close, pct] = g.compute_pre_pct(d.close, preclose, xdxr);
        *factor *= pct;
        Self {
            close,
            preclose,
            trading,
            xdxr,
            code: d.code,
            date: d.date,
            factor: *factor,
        }
    }
}
