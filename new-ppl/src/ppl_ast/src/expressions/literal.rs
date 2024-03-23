use std::fmt::Display;

use derive_more::From;
use salsa::DebugWithDb;

use crate::Db;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Literal {
    None,
    #[from]
    Boolean(bool),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;
        match self {
            None => write!(f, "none"),
            Boolean(b) => write!(f, "{b}"),
        }
    }
}

impl<DB: Sized + Db> DebugWithDb<DB> for Literal {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _db: &DB,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
