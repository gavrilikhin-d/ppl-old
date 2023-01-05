/// Trait for ranged things
pub trait Ranged {
    /// Get start of range
    fn start(&self) -> usize {
        self.range().start
    }

    /// Get end of range
    fn end(&self) -> usize {
        self.range().end
    }

    /// Get range
    fn range(&self) -> std::ops::Range<usize> {
        self.start()..self.end()
    }
}
