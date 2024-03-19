use derive_more::From;

#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: FunctionId,
    #[return_ref]
    pub name_parts: Vec<FunctionNamePart>,
}

#[salsa::interned]
pub struct FunctionId {
    #[return_ref]
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, From)]
pub enum FunctionNamePart {
    Text(Text),
    Parameter(Parameter),
}

#[salsa::interned]
pub struct Text {
    #[return_ref]
    pub text: String,
}

#[salsa::tracked]
pub struct Parameter {
    #[id]
    pub name: Identifier,
    pub ty: Typename,
}

#[salsa::interned]
pub struct Identifier {
    #[return_ref]
    pub text: String,
}

#[salsa::interned]
pub struct Typename {
    #[return_ref]
    pub text: String,
}
