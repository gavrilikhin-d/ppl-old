use crate::{
    declarations::{Function, FunctionId, Identifier, Parameter, Text, Typename},
    module::Module,
};

#[salsa::jar(db = Db)]
pub struct Jar(
    Module,
    Function,
    FunctionId,
    Text,
    Parameter,
    Identifier,
    Typename,
);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
