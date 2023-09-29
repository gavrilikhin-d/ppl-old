use crate::hir::{Expression, Function, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
use std::ops::Range;

/// AST for function call
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Call {
    /// Range of function call
    pub range: Range<usize>,
    /// Called function
    pub function: Function,
    /// Generic version of called function
    pub generic: Option<Function>,
    /// Arguments to the function call
    pub args: Vec<Expression>,
}

impl Ranged for Call {
    fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}

impl Typed for Call {
    fn ty(&self) -> Type {
        self.function.return_type().clone()
    }
}

impl Mutable for Call {
    fn is_mutable(&self) -> bool {
        self.ty().is_mutable()
    }
}
