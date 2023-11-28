use crate::syntax::StringWithOffset;

use super::Ranged;

use derive_more::{From, Into};

/// Escaped or unescaped identifier
#[derive(Debug, PartialEq, Eq, Clone, From, Into)]
pub struct Identifier(StringWithOffset);

impl Identifier {
    /// Get identifier as string slice
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl Ranged for Identifier {
    fn range(&self) -> std::ops::Range<usize> {
        self.0.range()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0
            .as_str()
            .trim_start_matches('`')
            .trim_end_matches('`')
    }
}

impl PartialEq<&str> for Identifier {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
