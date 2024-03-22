#[salsa::interned]
pub struct Identifier {
    #[return_ref]
    pub text: String,
}
