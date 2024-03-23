use std::fmt::Display;

use derive_more::From;
use rug::ops::Pow;
use rug::rational::ParseRationalError;
use salsa::DebugWithDb;

use crate::Db;

pub use rug::Integer;
pub use rug::Rational;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Literal {
    None,
    Boolean(bool),
    Integer(rug::Integer),
    Rational(rug::Rational),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;
        match self {
            None => write!(f, "none"),
            Boolean(b) => write!(f, "{b}"),
            Integer(i) => write!(f, "{i}"),
            Rational(r) => write!(f, "{}", maybe_to_decimal_string(r)),
            String(s) => write!(f, "{:?}", s),
        }
    }
}

impl<DB: Sized + Db> DebugWithDb<DB> for Literal {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _db: &DB,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

/// Trait to create rational number from decimal
pub trait FromDecimal {
    fn from_decimal(str: &str) -> Result<rug::Rational, ParseRationalError>;
}

impl FromDecimal for rug::Rational {
    fn from_decimal(str: &str) -> Result<rug::Rational, ParseRationalError> {
        if let Some(dot_offset) = str.find('.') {
            let str = format!(
                "{}{}/1{}",
                &str[..dot_offset],
                &str[(dot_offset + 1)..],
                format!("{:0^1$}", "", str.len() - dot_offset - 1)
            );
            return str.parse::<rug::Rational>();
        }

        str.parse::<rug::Rational>()
    }
}

/// Convert rational to decimal string, if it's exact
pub fn maybe_to_decimal_string(r: &rug::Rational) -> String {
    let mut denom = r.denom().clone();
    let pow2 = denom.remove_factor_mut(&Integer::from(2));
    let pow5 = denom.remove_factor_mut(&Integer::from(5));
    if denom != Integer::from(1) {
        return r.to_string();
    }

    let pow10 = pow2.max(pow5);
    let mut numer = r.numer().clone();
    numer *= Integer::from(2).pow(pow10 - pow2);
    numer *= Integer::from(5).pow(pow10 - pow5);

    let pow10 = pow10 as usize;
    let numer = format!("{numer:0>pow10$}");
    let dotpoint = numer.len() - pow10;
    let mut before_dot = &numer[..dotpoint];
    if before_dot.is_empty() {
        before_dot = "0";
    }
    let mut after_dot = &numer[dotpoint..];
    if after_dot.is_empty() {
        after_dot = "0";
    }
    format!("{before_dot}.{after_dot}")
}

#[cfg(test)]
mod test {
    #[test]
    fn to_decimal_string() {
        use super::maybe_to_decimal_string;

        let r = rug::Rational::from((1, 3));
        assert_eq!(maybe_to_decimal_string(&r), "1/3");

        let r = rug::Rational::from((1, 2));
        assert_eq!(maybe_to_decimal_string(&r), "0.5");

        let r = rug::Rational::from((5, 1));
        assert_eq!(maybe_to_decimal_string(&r), "5.0");

        let r = rug::Rational::from((5, 2));
        assert_eq!(maybe_to_decimal_string(&r), "2.5");

        let r = rug::Rational::from((1, 4));
        assert_eq!(maybe_to_decimal_string(&r), "0.25");

        let r = rug::Rational::from((1, 8));
        assert_eq!(maybe_to_decimal_string(&r), "0.125");

        let r = rug::Rational::from((1, 16));
        assert_eq!(maybe_to_decimal_string(&r), "0.0625");
    }
}
