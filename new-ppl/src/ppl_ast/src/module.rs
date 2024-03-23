use crate::{display::DisplayWithDb, statements::AnnotatedStatement, Db};

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<AnnotatedStatement>,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Module {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in self.statements(db) {
            writeln!(f, "{}", stmt.display_with(db))?;
        }
        Ok(())
    }
}
