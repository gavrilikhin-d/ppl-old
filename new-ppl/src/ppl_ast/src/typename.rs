#[salsa::interned]
pub struct Typename {
    #[return_ref]
    pub text: String,
}
