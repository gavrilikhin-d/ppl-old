#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: FunctionId,
}

#[salsa::interned]
pub struct FunctionId {
    #[return_ref]
    pub text: String,
}
