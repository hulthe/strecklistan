use regex::Regex;
use std::fmt::{self, Display};
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct Currency(i32);

impl Currency {
    pub fn fractional(self) -> i32 {
        self.0 % 100
    }

    pub fn whole(self) -> i32 {
        self.0 / 100
    }

    pub fn as_f64(self) -> f64 {
        self.whole() as f64 + self.fractional() as f64 / 100.0
    }
}

impl Add for Currency {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Currency(self.0 + other.0)
    }
}

impl AddAssign for Currency {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for Currency {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Currency(self.0 - other.0)
    }
}

impl SubAssign for Currency {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Neg for Currency {
    type Output = Self;
    fn neg(self) -> Self {
        Currency(-self.0)
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 0 {
            write!(f, "-")?;
        }
        write!(f, "{}", self.whole().abs())?;
        if self.fractional() != 0 {
            write!(f, ".{:02}", self.fractional().abs())?;
        }
        Ok(())
    }
}

lazy_static! {
    static ref CURRENCY_RE: Regex =
        Regex::new(r"^(?P<neg>-)?\s*?(?P<whole>\d+)(\.(?P<frac>\d+))?$").unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyParseError {
    FracGreaterThan100,
    MatchFailed,
}

impl FromStr for Currency {
    type Err = CurrencyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(captures) = CURRENCY_RE.captures(s) {
            let neg = captures.name("neg").is_some();
            let whole: i32 = captures.name("whole").unwrap().as_str().parse().unwrap();

            let frac_s = captures.name("frac").map(|f| f.as_str()).unwrap_or("00");

            let mut frac: i32 = frac_s.parse().unwrap_or(0);
            if frac_s.len() == 1 {
                frac *= 10;
            }

            if frac >= 100 || frac < 0 {
                return Err(CurrencyParseError::FracGreaterThan100);
            }

            if neg {
                Ok(Currency(-(whole * 100 + frac)))
            } else {
                Ok(Currency(whole * 100 + frac))
            }
        } else {
            Err(CurrencyParseError::MatchFailed)
        }
    }
}

impl From<i32> for Currency {
    fn from(other: i32) -> Self {
        Currency(other)
    }
}

impl Into<i32> for Currency {
    fn into(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_currency_parsing() {
        assert_eq!("123.123".parse::<Currency>(), Err(CurrencyParseError::FracGreaterThan100));
        assert_eq!("123.-3".parse::<Currency>(), Err(CurrencyParseError::MatchFailed));
        assert_eq!("-123.-23".parse::<Currency>(), Err(CurrencyParseError::MatchFailed));
        assert_eq!("123.-123".parse::<Currency>(), Err(CurrencyParseError::MatchFailed));
        assert_eq!("0.0.0".parse::<Currency>(), Err(CurrencyParseError::MatchFailed));

        for i in (-9999..9999).step_by(9) {
            let f = format!("{}", Currency(i));
            println!("{}", f);
            assert_eq!(f.parse(), Ok(Currency(i)));
        }
    }

    #[test]
    fn test_currency_add_subtract() {
        for x in -999..999 {
            for y in -999..999 {
                let a: Currency = x.into();
                let b: Currency = y.into();

                assert_eq!(a - a, 0.into());
                assert_eq!(b - b, 0.into());
                assert_eq!(a + a - a, a);
                assert_eq!(b + a - b, a);
                assert_eq!(a + b - b, a);
                assert_eq!(b + b - b, b);
                assert_eq!(a + b - a, b);
                assert_eq!(b + a - a, b);

                let mut a2: Currency = a;

                a2 += a;
                assert_eq!(a2, a + a);
                a2 -= a;
                assert_eq!(a2, a);
                a2 += b;
                assert_eq!(a2, a + b);
                a2 += b;
                assert_eq!(a2, a + b + b);
                a2 -= b + a;
                assert_eq!(a2, b);
            }
        }
    }

    #[test]
    fn test_currency_f64_repr() {
        assert_eq!(Currency::from(3220).as_f64(), 32.20);
        assert_eq!(Currency::from(9999999).as_f64(), 99999.99);
        assert_eq!(Currency::from(0).as_f64(), 0.0);
        assert_eq!(Currency::from(1).as_f64(), 0.01);
        assert_eq!(Currency::from(-232323).as_f64(), -2323.23);
        assert_eq!(Currency::from(-1).as_f64(), -0.01);
    }
}
