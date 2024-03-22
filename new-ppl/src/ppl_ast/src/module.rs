use crate::declarations::Declaration;

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<Declaration>,
}
