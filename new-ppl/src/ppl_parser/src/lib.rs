#![feature(trait_upcasting)]
#![feature(debug_closure_helpers)]

use diagnostic::Diagnostics;
use parser::parse_module;
use source::SourceProgram;

pub mod diagnostic;
pub mod parser;
pub mod source;

#[salsa::jar(db = Db)]
pub struct Jar(SourceProgram, parse_module, Diagnostics);

pub trait Db: salsa::DbWithJar<Jar> + ppl_ast::Db {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + ppl_ast::Db {}
