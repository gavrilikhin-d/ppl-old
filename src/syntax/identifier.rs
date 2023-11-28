use std::{fmt::Display, ops::Deref};

use crate::syntax::StringWithOffset;

use super::Ranged;

use derive_more::{From, Into};

/// Escaped or unescaped identifier
#[derive(Debug, PartialEq, Eq, Clone, Hash, From, Into)]
pub struct Identifier(StringWithOffset);

impl Identifier {
    /// Get identifier as string slice
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    /// Move identifier to specified offset
    pub fn at(self, offset: usize) -> Self {
        Self(self.0.at(offset))
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

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<&str> for Identifier {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
