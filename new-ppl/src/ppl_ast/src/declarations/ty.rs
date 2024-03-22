use crate::typename::Typename;

#[salsa::tracked]
pub struct Type {
    #[id]
    pub name: Typename,
}
