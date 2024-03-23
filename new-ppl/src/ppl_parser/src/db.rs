use crate::{diagnostic::Diagnostics, parser::module, source::SourceProgram};

#[salsa::jar(db = Db)]
pub struct Jar(SourceProgram, module, Diagnostics);

pub trait Db: salsa::DbWithJar<Jar> + ppl_ast::Db {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + ppl_ast::Db {}
