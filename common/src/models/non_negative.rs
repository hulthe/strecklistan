use crate::currency::{Currency, CurrencyParseError};
use std::str::FromStr;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct AbsCurrency(Currency);

impl Into<Currency> for AbsCurrency {
    fn into(self) -> Currency {
        self.0
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
