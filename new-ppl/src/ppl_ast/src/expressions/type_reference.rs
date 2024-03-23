use salsa::DebugWithDb;

use crate::{display::DisplayWithDb, typename::Typename, Db};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeReference {
    pub ty: Typename,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for TypeReference {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty.display_with(db))
    }
}

impl<DB: Sized + Db> DebugWithDb<DB> for TypeReference {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        write!(f, "{}", self.display_with(db))
    }
}
