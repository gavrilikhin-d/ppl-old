use std::{
    fs, io,
    path::{Path, PathBuf},
};

use miette::{MietteError, NamedSource, SourceCode, SpanContents};

/// Wrapper around [`PathBuf`] that implements [`SourceCode`]
#[derive(Debug)]
pub struct SourceFile {
    /// Path to file
    path: PathBuf,
    /// File contents
    source: NamedSource,
}

impl Clone for SourceFile {
    fn clone(&self) -> Self {
        Self::with_path(&self.path).unwrap()
    }
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl SourceFile {
    /// Wrap path to source file
    pub fn with_path(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(&path)?;
        Ok(Self {
            path,
            source: NamedSource::new(name, source),
        })
    }

    /// Get path to the source file
    pub fn path(&self) -> &Path {
        &self.path
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
