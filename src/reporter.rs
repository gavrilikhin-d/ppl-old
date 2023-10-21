use std::fmt::{self, Display};

use miette::{Diagnostic, LabeledSpan, MietteHandler, ReportHandler, SourceCode};

/// Struct to report errors
pub struct Reporter;

impl Default for Reporter {
    fn default() -> Self {
        Self
    }
}

impl ReportHandler for Reporter {
    fn debug(&self, error: &(dyn miette::Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(error, f);
        }

        let handler = MietteHandler::default();
        // Check that this is an error vector.
        // We want to threat it as just a collection of unrelated errors
        if error.to_string().is_empty() {
            if let Some(source_code) = error.source_code() {
                for e in error.related().unwrap() {
                    handler.debug(
                        &WithSourceCode {
                            diagnostic: e,
                            source_code,
                        },
                        f,
                    )?;
                }
            } else {
                for e in error.related().unwrap() {
                    handler.debug(e, f)?;
                }
            }
            Ok(())
        } else {
            handler.debug(error, f)
        }
    }
}

struct WithSourceCode<'d, 's> {
    diagnostic: &'d dyn Diagnostic,
    source_code: &'s dyn SourceCode,
}

impl Display for WithSourceCode<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic)
    }
}

impl fmt::Debug for WithSourceCode<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Debug::fmt(&self.diagnostic, f)
    }
}

impl std::error::Error for WithSourceCode<'_, '_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.diagnostic.source()
    }
}

impl Diagnostic for WithSourceCode<'_, '_> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.code()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.diagnostic.diagnostic_source()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.help()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.diagnostic.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.diagnostic.related()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.diagnostic.severity()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.diagnostic.source_code().or(Some(self.source_code))
    }
}
