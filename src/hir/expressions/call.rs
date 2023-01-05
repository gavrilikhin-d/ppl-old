use crate::hir::{Expression, FunctionDeclaration, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
use std::ops::Range;
use std::sync::Arc;

/// AST for function call
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Call {
    /// Range of function call
    pub range: Range<usize>,
    /// Called function
    pub function: Arc<FunctionDeclaration>,
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
        self.function.return_type.clone()
    }
}

impl Mutable for Call {
    fn is_mutable(&self) -> bool {
        self.function.return_type.is_mutable()
    }
}
