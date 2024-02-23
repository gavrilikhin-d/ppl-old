use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use thiserror::Error;

use miette::{MietteError, NamedSource, SourceCode, SpanContents};

/// Wrapper around [`PathBuf`] that implements [`SourceCode`]
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path to file
    path: PathBuf,
    /// File contents
    source: Arc<NamedSource<String>>,
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.source.name() == other.source.name()
    }
}

impl Eq for SourceFile {}

impl SourceFile {
    /// Get virtual source file
    pub fn in_memory(source: NamedSource<String>) -> Self {
        Self {
            path: "<memory>".into(),
            source: Arc::new(source),
        }
    }

    /// Wrap path to source file
    pub fn with_path(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let name = path
            .file_name()
            .expect(format!("Can't get filename of `{}`", path.display()).as_str())
            .to_string_lossy()
            .to_string();
        let source = fs::read_to_string(&path)?;
        Ok(Self {
            path,
            source: Arc::new(NamedSource::new(name, source)),
        })
    }

    /// Get path to the source file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Name of the source file
    pub fn name(&self) -> &str {
        self.source.name()
    }

    /// Line number for byte index
    pub fn line_number(&self, offset: usize) -> LineNumber {
        let str = self.source.inner();
        let end = offset.min(str.len());
        let lines = str[..end].chars().filter(|&c| c == '\n').count();
        LineNumber::from_zero_based(lines)
    }

    /// Column number for byte index
    pub fn column_number(&self, offset: usize) -> ColumnNumber {
        let str = self.source.inner();
        let end = offset.min(str.len());
        let last_line = str[..end].rfind('\n').map_or(0, |i| i + 1);
        ColumnNumber::from_zero_based(end - last_line)
    }
}

impl SourceCode for SourceFile {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        self.source
            .read_span(span, context_lines_before, context_lines_after)
    }
}

/// A line number
pub struct LineNumber(usize);

impl LineNumber {
    /// Create a new `LineNumber` from a 0-based index
    pub fn from_zero_based(n: usize) -> Self {
        Self(n)
    }

    /// Create a new `LineNumber` from a 1-based index
    pub fn from_one_based(n: usize) -> Result<Self, ZeroAsOneBased> {
        if n == 0 {
            return Err(ZeroAsOneBased);
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

/// A column number
pub struct ColumnNumber(usize);

impl ColumnNumber {
    /// Create a new `ColumnNumber` from a 0-based index
    pub fn from_zero_based(n: usize) -> Self {
        Self(n)
    }

    /// Create a new `ColumnNumber` from a 1-based index
    pub fn from_one_based(n: usize) -> Result<Self, ZeroAsOneBased> {
        if n == 0 {
            return Err(ZeroAsOneBased);
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

/// Diagnostic for invalid 1-based line/column number
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("'0' is not a valid 1-based number")]
pub struct ZeroAsOneBased;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_number() {
        let content = "Hello\nworld!";
        let n = content.len();
        let source_file =
            SourceFile::in_memory(NamedSource::new("test".to_string(), content.to_string()));
        assert_eq!(source_file.line_number(0).zero_based(), 0);
        assert_eq!(source_file.line_number(4).zero_based(), 0);
        // '\n' is included in the line
        assert_eq!(source_file.line_number(5).zero_based(), 0);
        assert_eq!(source_file.line_number(6).zero_based(), 1);
        assert_eq!(source_file.line_number(n - 1).zero_based(), 1);
        assert_eq!(source_file.line_number(n).zero_based(), 1);
        assert_eq!(source_file.line_number(n + 1).zero_based(), 1);

        assert_eq!(source_file.line_number(0).one_based(), 1);
        assert_eq!(source_file.line_number(4).one_based(), 1);
        // '\n' is included in the line
        assert_eq!(source_file.line_number(5).one_based(), 1);
        assert_eq!(source_file.line_number(6).one_based(), 2);
        assert_eq!(source_file.line_number(n - 1).one_based(), 2);
        assert_eq!(source_file.line_number(n).one_based(), 2);
        assert_eq!(source_file.line_number(n + 1).one_based(), 2);
    }

    #[test]
    fn column_number() {
        let content = "Hello\nworld!";
        let n = content.len();
        let source_file =
            SourceFile::in_memory(NamedSource::new("test".to_string(), content.to_string()));
        assert_eq!(source_file.column_number(0).zero_based(), 0);
        assert_eq!(source_file.column_number(4).zero_based(), 4);
        // '\n' is included in the line
        assert_eq!(source_file.column_number(5).zero_based(), 5);
        assert_eq!(source_file.column_number(6).zero_based(), 0);
        assert_eq!(source_file.column_number(n - 1).zero_based(), 5);
        assert_eq!(source_file.column_number(n).zero_based(), 6);
        assert_eq!(source_file.column_number(n + 1).zero_based(), 6);

        assert_eq!(source_file.column_number(0).one_based(), 1);
        assert_eq!(source_file.column_number(4).one_based(), 5);
        // '\n' is included in the line
        assert_eq!(source_file.column_number(5).one_based(), 6);
        assert_eq!(source_file.column_number(6).one_based(), 1);
        assert_eq!(source_file.column_number(n - 1).one_based(), 6);
        assert_eq!(source_file.column_number(n).one_based(), 7);
        assert_eq!(source_file.column_number(n + 1).one_based(), 7);
    }
}
