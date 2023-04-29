use nom::IResult;
use thiserror::Error;

/// Creates recoverable error for nom
#[macro_export]
macro_rules! err {
    ($error: expr) => {
        Err(nom::Err::Error($error))
    };
}

/// Creates boxed recoverable error for nom
#[macro_export]
macro_rules! err_boxed {
    ($error: expr) => {
        Err(nom::Err::Error(Box::new($error)))
    };
}

/// Creates boilerplate for error enum
#[macro_export]
macro_rules! error_enum {
	($name:ident, $($variant:ident)|*) => {
		#[derive(Debug, thiserror::Error, derive_more::From, PartialEq, Eq)]
		pub enum $name<'i> {
			$(
				#[error(transparent)]
				$variant($variant<'i>),
			)*
		}
	};
}

/// Helper function to easily map errors
pub fn map_err<I, O, E1, E2>(res: IResult<I, O, E1>, f: impl Fn(E1) -> E2) -> IResult<I, O, E2> {
    res.map_err(|e| e.map(f))
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Regex didn't match")]
pub struct RegexMismatch {}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected {expected:?}")]
pub struct Expected<'i> {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    pub at: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected one of {variants:?}")]
pub struct ExpectedOneOf<'i> {
    /// What was expected
    pub variants: Vec<String>,
    /// Where the error occurred
    pub at: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected matching {expected:?} to match {to_match:?}")]
pub struct ExpectedMatching<'i> {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    pub at: &'i str,
    /// What was expected to match
    pub to_match: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("referencing unknown rule {name:?}")]
pub struct UnknownRuleReference<'i> {
    /// Unknown rule name
    pub name: &'i str,
}
