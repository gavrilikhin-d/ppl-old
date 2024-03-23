use crate::{display::DisplayWithDb, Db};

use self::{literal::Literal, type_reference::TypeReference};

pub mod literal;
pub mod type_reference;

use derive_more::From;
use salsa::DebugWithDb;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Expression {
    Literal(Literal),
    TypeReference(TypeReference),
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Expression {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expression::*;
        match self {
            Literal(l) => l.fmt_with(db, f),
            TypeReference(t) => t.fmt_with(db, f),
        }
    }
}

impl<DB: Sized + Db> DebugWithDb<DB> for Expression {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        use Expression::*;
        match self {
            Literal(d) => d.fmt(f, db, include_all_fields),
            TypeReference(t) => t.fmt(f, db, include_all_fields),
        }
    }
}
