use crate::{
    declarations::function::{Function, FunctionId, Text},
    identifier::Identifier,
    module::Module,
    typename::Typename,
};

#[salsa::jar(db = Db)]
pub struct Jar(Module, Function, FunctionId, Text, Identifier, Typename);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
