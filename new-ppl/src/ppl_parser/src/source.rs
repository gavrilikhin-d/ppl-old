#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub text: String,
}
