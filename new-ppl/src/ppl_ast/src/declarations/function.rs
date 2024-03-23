use derive_more::From;
use salsa::DebugWithDb;

use crate::{display::DisplayWithDb, identifier::Identifier, typename::Typename, Db};

#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: FunctionId,
    #[return_ref]
    pub name_parts: Vec<FunctionNamePart>,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Function {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fn {}",
            self.name_parts(db)
                .iter()
                .map(|p| p.to_string_with(db))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
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
                FunctionNamePart::Text(text) => text.to_string_with(db),
                FunctionNamePart::Parameter(param) => {
                    format!("<:{}>", param.ty.display_with(db))
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        FunctionId::new(db, text)
    }
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for FunctionId {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text(db))
    }
}

#[derive(Debug, PartialEq, Eq, From)]
pub enum FunctionNamePart {
    Text(Text),
    Parameter(Parameter),
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for FunctionNamePart {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FunctionNamePart::*;
        match self {
            Text(t) => t.fmt_with(db, f),
            Parameter(p) => p.fmt_with(db, f),
        }
    }
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
            Text(t) => DebugWithDb::fmt(t, f, db, include_all_fields),
            Parameter(p) => p.fmt(f, db, include_all_fields),
        }
    }
}

#[salsa::interned]
pub struct Text {
    #[return_ref]
    pub text: String,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Text {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text(db))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub ty: Typename,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Parameter {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}: {}>",
            self.name.display_with(db),
            self.ty.display_with(db)
        )
    }
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
