use std::ops::Range;

/// Value at some offset
#[derive(Debug, PartialEq)]
pub struct WithOffset<T> {
	/// Offset of the value
	pub offset: usize,
	/// Value at some offset
	pub value: T,
}

impl WithOffset<String> {
	/// Get range of the underlying string
	///
	/// # Example
	/// ```
	/// use ppl::syntax::WithOffset;
	///
	/// let value = WithOffset { offset: 0, value: "hello".to_string() };
	/// assert_eq!(value.range(), 0..5);
	/// ```
	pub fn range(&self) -> Range<usize> {
		self.offset..self.offset + self.value.len()
	}
}