use std::path::PathBuf;

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub path: Option<PathBuf>,
    #[return_ref]
    pub text: String,
}
