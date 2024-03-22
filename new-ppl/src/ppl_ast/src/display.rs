use std::fmt::{self, Display, Formatter, FormatterFn};

pub trait DisplayWithDb<'me, DB: ?Sized + 'me> {
    fn fmt_with(&self, db: &DB, f: &mut Formatter<'_>) -> fmt::Result;

    fn display_with(&self, db: &DB) -> FormatterFn<impl Fn(&mut Formatter<'_>) -> fmt::Result> {
        FormatterFn(|f| self.fmt_with(db, f))
    }

    fn to_string_with(&self, db: &DB) -> String {
        self.display_with(db).to_string()
    }
}

impl<'me, DB: ?Sized + 'me, D: Display> DisplayWithDb<'me, DB> for D {
    fn fmt_with(&self, _db: &DB, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
