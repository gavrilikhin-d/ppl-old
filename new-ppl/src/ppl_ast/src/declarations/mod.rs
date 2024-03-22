use derive_more::From;

use self::{function::Function, ty::Type};

pub mod function;
pub mod ty;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Declaration {
    Function(Function),
    Type(Type),
}
