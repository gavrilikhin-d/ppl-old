/// A trait for getting the offset of a subslice from the outer slice.
pub trait SubsliceOffset {
    /// Returns the offset of the inner string from the outer string.
    ///
    /// # Examples
    /// ```
    /// use syntax::SubsliceOffset;
    ///
    /// let string = "a\nb\nc";
    /// let lines: Vec<&str> = string.lines().collect();
    /// assert_eq!(string.subslice_offset(lines[0]), Some(0)); // &"a"
    /// assert_eq!(string.subslice_offset(lines[1]), Some(2)); // &"b"
    /// assert_eq!(string.subslice_offset(lines[2]), Some(4)); // &"c"
    /// assert_eq!(string.subslice_offset("other!"), None);
    /// ```
    fn subslice_offset(&self, inner: &str) -> Option<usize>;
}

impl SubsliceOffset for &str {
    fn subslice_offset(&self, inner: &str) -> Option<usize> {
        let start = inner.as_ptr() as usize;
        let end = start + inner.len();
        let self_start = self.as_ptr() as usize;
        let self_end = self_start + self.len();
        if start >= self_start && end <= self_end {
            Some(start - self_start)
        } else {
            None
        }
    }
}
