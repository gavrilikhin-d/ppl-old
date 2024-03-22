use crate::declarations::function::Function;

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<Function>,
}
