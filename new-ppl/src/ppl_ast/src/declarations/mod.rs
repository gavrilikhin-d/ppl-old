use derive_more::From;

use crate::Db;

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

impl FunctionId {
    pub fn from_parts(db: &dyn Db, parts: &[FunctionNamePart]) -> Self {
        let text = parts
            .iter()
            .map(|part| match part {
                FunctionNamePart::Text(text) => text.text(db).clone(),
                FunctionNamePart::Parameter(param) => {
                    format!("<:{}>", param.ty.text(db))
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        FunctionId::new(db, text)
    }
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

#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
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
