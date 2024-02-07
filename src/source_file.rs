use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

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
            path: PathBuf::new(),
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
