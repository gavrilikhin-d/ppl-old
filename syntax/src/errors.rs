use derive_more::From;
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
macro_rules! error_enum {
	($name:ident, $($variant:ident)|*) => {
		#[derive(Debug, Error, From, PartialEq, Eq)]
		pub enum $name<'i> {
			$(
				#[error(transparent)]
				$variant($variant<'i>),
			)*
		}
	};
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Regex didn't match")]
pub struct RegexMismatch {}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected rule name")]
pub struct ExpectedRuleName<'i> {
    /// The input that caused the error
    pub at: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("lowercase rule name")]
pub struct LowercaseRuleName<'i> {
    /// The rule name that caused the error
    pub name: &'i str,
}

error_enum!(RuleNameError, ExpectedRuleName | LowercaseRuleName);

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown rule reference")]
pub struct UnknownRuleReference<'i> {
    /// The rule name that caused the error
    pub name: &'i str,
}

error_enum!(RuleReferenceError, RuleNameError | UnknownRuleReference);

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected regex")]
pub struct ExpectedRegex<'i> {
    /// The input that caused the error
    pub at: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid regex: ${reason}")]
pub struct InvalidRegex<'i> {
    /// Location of the invalid regex
    pub re: &'i str,
    /// The reason why the regex is invalid
    pub reason: String,
}

error_enum!(RegexError, ExpectedRegex | InvalidRegex);
