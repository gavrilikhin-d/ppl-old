use crate::hir::{Expression, FunctionDeclaration, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
use std::ops::Range;
use std::sync::Weak;

/// Kind of function call
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CallKind {
	Operation,
	Call
}

/// AST for function call
#[derive(Debug, Clone)]
pub struct Call {
    /// Range of function call
    pub range: Range<usize>,
    /// Called function
    pub function: Weak<FunctionDeclaration>,
    /// Arguments to the function call
    pub args: Vec<Expression>,
}

impl PartialEq for Call {
	fn eq(&self, other: &Self) -> bool {
		self.range == other.range &&
		self.function.ptr_eq(&other.function) &&
		self.args == other.args
	}
}
impl Eq for Call {}

impl Ranged for Call {
    fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}

impl Typed for Call {
    fn ty(&self) -> Type {
        self.function.upgrade().unwrap().return_type.clone()
    }
}

impl Mutable for Call {
    fn is_mutable(&self) -> bool {
        self.ty().is_mutable()
    }
}
