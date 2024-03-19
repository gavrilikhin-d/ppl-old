use std::fmt::{self, Display, Formatter};

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
