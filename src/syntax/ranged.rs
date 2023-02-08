use enum_dispatch::enum_dispatch;

/// Trait for ranged things
#[enum_dispatch]
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

impl<T: Ranged> Ranged for Vec<T> {
	fn range(&self) -> std::ops::Range<usize> {
		self.first().map(|x| x.start()).unwrap_or(0)..self.last().map(|x| x.end()).unwrap_or(0)
	}
}