use std::fmt;

use miette::{MietteHandler, ReportHandler};

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
            for e in error.related().unwrap() {
                handler.debug(e, f)?;
            }
            Ok(())
        } else {
            handler.debug(error, f)
        }
    }
}
