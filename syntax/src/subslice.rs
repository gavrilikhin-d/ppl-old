use std::ops::Range;

/// A trait for getting the offset of a subslice from the outer slice.
pub trait SubsliceOffset {
    /// Returns the offset of the inner string in the outer string.
    ///
    /// # Examples
    /// ```
    /// use syntax::SubsliceOffset;
    ///
    /// let string = "a\nb\nc";
    /// let lines: Vec<&str> = string.lines().collect();
    /// assert_eq!(lines[0].offset_in(string), Some(0)); // &"a"
    /// assert_eq!(lines[1].offset_in(string), Some(2)); // &"b"
    /// assert_eq!(lines[2].offset_in(string), Some(4)); // &"c"
    /// assert_eq!("other!".offset_in(string), None);
    /// ```
    fn offset_in(&self, outer: &str) -> Option<usize>;

    /// Returns the range of the inner string in the outer string.
    ///
    /// # Examples
    /// ```
    /// use syntax::SubsliceOffset;
    ///
    /// let string = "a\nb\nc";
    /// let lines: Vec<&str> = string.lines().collect();
    /// assert_eq!(lines[0].range_in(string), Some(0..1)); // &"a"
    /// assert_eq!(lines[1].range_in(string), Some(2..3)); // &"b"
    /// assert_eq!(lines[2].range_in(string), Some(4..5)); // &"c"
    /// assert_eq!("other!".range_in(string), None);
    /// ```
    fn range_in(&self, outer: &str) -> Option<Range<usize>>;
}

impl SubsliceOffset for &str {
    fn offset_in(&self, outer: &str) -> Option<usize> {
        let outer_start = outer.as_ptr() as usize;
        let outer_end = outer_start + outer.len();

        let start = self.as_ptr() as usize;
        let end = start + self.len();

        if start >= outer_start && end <= outer_end {
            Some(start - outer_start)
        } else {
            None
        }
    }

    fn range_in(&self, outer: &str) -> Option<Range<usize>> {
        self.offset_in(outer)
            .map(|offset| offset..offset + self.len())
    }
}
