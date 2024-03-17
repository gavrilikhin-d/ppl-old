#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone)]
pub struct Diagnostic {
    pub message: String,
}
