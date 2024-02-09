use thiserror::Error;

/// A line number
pub struct LineNumber(usize);

impl LineNumber {
    /// Create a new `LineNumber` from a 0-based index
    pub fn from_zero_based(n: usize) -> Self {
        Self(n)
    }

    /// Create a new `LineNumber` from a 1-based index
    pub fn from_one_based(n: usize) -> Result<Self, ZeroAsOneBasedLineNumber> {
        if n == 0 {
            return Err(ZeroAsOneBasedLineNumber)
        }
        Ok(Self(n - 1))
    }

    /// Get the 0-based index
    pub fn zero_based(&self) -> usize {
        self.0
    }

    /// Get the 1-based index
    pub fn one_based(&self) -> usize {
        self.0 + 1
    }
}

/// Diagnostic for invalid 1-based line number
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("'0' is not a valid 1-based line number")]
pub struct ZeroAsOneBasedLineNumber;