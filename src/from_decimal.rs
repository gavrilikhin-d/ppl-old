use rug::rational::ParseRationalError;

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