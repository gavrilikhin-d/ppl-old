use crate::{display::DisplayWithDb, identifier::Identifier, Db};

#[salsa::tracked]
pub struct Annotation {
    name: Identifier,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Annotation {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.name(db).display_with(db))
    }
}
