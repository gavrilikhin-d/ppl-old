use crate::Db;

use self::literal::Literal;

pub mod literal;

use derive_more::{Display, From};
use salsa::DebugWithDb;

#[derive(Debug, PartialEq, Eq, Clone, From, Display)]
pub enum Expression {
    Literal(Literal),
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
        }
    }
}
