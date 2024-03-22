use derive_more::From;
use salsa::DebugWithDb;

use crate::{identifier::Identifier, typename::Typename, Db};

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

impl<DB: Sized + Db> DebugWithDb<DB> for FunctionNamePart {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        use FunctionNamePart::*;
        match self {
            Text(t) => t.fmt(f, db, include_all_fields),
            Parameter(p) => p.fmt(f, db, include_all_fields),
        }
    }
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

impl<DB: Sized + Db> DebugWithDb<DB> for Parameter {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        f.debug_struct("Parameter")
            .field("name", &self.name.debug_with(db, include_all_fields))
            .field("ty", &self.ty.debug_with(db, include_all_fields))
            .finish()
    }
}
