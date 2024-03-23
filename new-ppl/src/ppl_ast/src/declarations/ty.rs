use crate::{display::DisplayWithDb, typename::Typename, Db};

#[salsa::tracked]
pub struct Type {
    #[id]
    pub name: Typename,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Type {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {}", self.name(db).display_with(db))
    }
}
