use crate::declarations::Function;

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<Function>,
}
