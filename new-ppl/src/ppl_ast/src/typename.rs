use crate::{display::DisplayWithDb, Db};

#[salsa::interned]
pub struct Typename {
    #[return_ref]
    pub text: String,
}

impl<'me> DisplayWithDb<'me, dyn Db + 'me> for Typename {
    fn fmt_with(&self, db: &dyn Db, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text(db))
    }
}
