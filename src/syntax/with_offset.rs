use std::ops::Range;

use super::Ranged;

/// Value at some offset
#[derive(Debug, PartialEq, Clone)]
pub struct WithOffset<T> {
	/// Offset of the value
	pub offset: usize,
	/// Value at some offset
	pub value: T,
}

impl Ranged for WithOffset<String> {
	/// Get range of the underlying string
	///
	/// # Example
	/// ```
	/// use ppl::syntax::WithOffset;
	///
	/// let value = WithOffset { offset: 0, value: "hello".to_string() };
	/// assert_eq!(value.range(), 0..5);
	/// ```
	fn range(&self) -> Range<usize> {
		self.offset..self.offset + self.value.len()
	}
}