use std::fmt::{self, Display, Formatter, FormatterFn};

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub message: String,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait DisplayDiagnostics {
    fn display(&self) -> impl Display;
}

impl DisplayDiagnostics for Vec<Diagnostic> {
    fn display(&self) -> impl Display {
        FormatterFn(move |f| {
            for diagnostic in self {
                writeln!(f, "{}", diagnostic)?;
            }
            Ok(())
        })
    }
}
