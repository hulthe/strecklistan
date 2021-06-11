use crate::currency::{Currency, CurrencyParseError};
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::str::FromStr;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct AbsCurrency(Currency);

impl TryFrom<Currency> for AbsCurrency {
    type Error = &'static str;

    fn try_from(value: Currency) -> Result<Self, Self::Error> {
        if value < 0.into() {
            Err("currency less than 0")
        } else {
            Ok(AbsCurrency(value))
        }
    }
}

impl From<AbsCurrency> for Currency {
    fn from(val: AbsCurrency) -> Self {
        val.0
    }
}

impl FromStr for AbsCurrency {
    type Err = CurrencyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let currency = Currency::from_str(s)?;
        if currency < 0.into() {
            Err(CurrencyParseError::MatchFailed)
        } else {
            Ok(AbsCurrency(currency))
        }
    }
}

impl Display for AbsCurrency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
