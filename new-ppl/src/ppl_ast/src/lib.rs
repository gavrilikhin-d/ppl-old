use declarations::{Function, FunctionId};
use module::Module;

pub mod declarations;
pub mod module;

#[salsa::jar(db = Db)]
pub struct Jar(Module, Function, FunctionId);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
