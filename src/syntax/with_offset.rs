use std::{ops::{Range, Deref}, fmt::Display};

use super::Ranged;

/// Value with starting offset
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WithOffset<T> {
	/// Offset of the start of the value
	pub offset: usize,
	/// Value at an offset
	pub value: T,
}

/// String at some offset
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringWithOffset {
	/// Offset of the start of the string
	offset: usize,
	/// String value
	value: String,
}

impl StringWithOffset {
	/// Reposition string at some offset
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{StringWithOffset, Ranged};
	///
	/// let value = StringWithOffset::from("hello").at(10);
	/// assert_eq!(value.range(), 10..15);
	/// ```
	pub fn at(self, offset: usize) -> Self {
		Self { offset, ..self }
	}
}

impl From<&str> for StringWithOffset {
	/// Create string at offset 0
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{StringWithOffset, Ranged};
	///
	/// let value = StringWithOffset::from("hello");
	/// assert_eq!(value.range(), 0..5);
	/// ```
	fn from(value: &str) -> Self {
		Self {
			offset: 0,
			value: value.to_string(),
		}
	}
}

impl From<String> for StringWithOffset {
	/// Create string at offset 0
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{StringWithOffset, Ranged};
	///
	/// let value = StringWithOffset::from("hello".to_string());
	/// assert_eq!(value.range(), 0..5);
	/// ```
	fn from(value: String) -> Self {
		Self {
			offset: 0,
			value,
		}
	}
}

impl Deref for StringWithOffset {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl Display for StringWithOffset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}

impl From<StringWithOffset> for String {
	fn from(value: StringWithOffset) -> Self {
		value.value
	}
}

impl From<&StringWithOffset> for String {
	fn from(value: &StringWithOffset) -> Self {
		value.value.clone()
	}
}

impl Ranged for StringWithOffset {
	/// Get range of the underlying string
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{StringWithOffset, Ranged};
	///
	/// let value = StringWithOffset::from("hello").at(0);
	/// assert_eq!(value.range(), 0..5);
	/// ```
	fn range(&self) -> Range<usize> {
		self.offset..self.offset + self.value.len()
	}
}

impl PartialEq<&str> for StringWithOffset {
	fn eq(&self, other: &&str) -> bool {
		self.value == *other
	}
}