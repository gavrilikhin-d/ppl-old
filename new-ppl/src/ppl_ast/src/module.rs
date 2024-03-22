use crate::statements::AnnotatedStatement;

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<AnnotatedStatement>,
}
