use miette::SourceSpan;

use crate::SourceFile;

/// Location inside a source file
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Source file this location is in. `None` means current file
    pub source_file: Option<SourceFile>,
    /// Span of the source file
    pub at: SourceSpan,
}

impl<S: Into<SourceSpan>> From<S> for SourceLocation {
    fn from(value: S) -> Self {
        SourceLocation {
            source_file: None,
            at: value.into(),
        }
    }
}

/// Some value with a source location
#[derive(Debug)]
pub struct WithSourceLocation<T> {
    /// The value itself
    pub value: T,
    /// Source location of the value
    pub source_location: SourceLocation,
}

impl<T: Clone> Clone for WithSourceLocation<T> {
    fn clone(&self) -> Self {
        WithSourceLocation {
            value: self.value.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

/// Trait to add source location to a value
pub trait AddSourceLocation {
    fn at(self, source_location: impl Into<SourceLocation>) -> WithSourceLocation<Self>
    where
        Self: Sized,
    {
        WithSourceLocation {
            value: self,
            source_location: source_location.into(),
        }
    }
}
