use crate::hir::{Expression, Function, FunctionNamePart, Generic, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
use std::fmt::Display;
use std::ops::Range;
use std::sync::Arc;

/// AST for function call
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Call {
    /// Range of function call
    pub range: Range<usize>,

    /// Called function
    pub function: Arc<Function>,
    /// Generic version of called function
    pub generic: Option<Arc<Function>>,

    /// Arguments to the function call
    pub args: Vec<Expression>,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut arg = self.args.iter();

        write!(
            f,
            "{}",
            self.function
                .name_parts()
                .iter()
                .map(|part| match part {
                    FunctionNamePart::Text(text) => text.to_string(),
                    FunctionNamePart::Parameter(_) => arg.next().unwrap().to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
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
        self.ty().is_mutable()
    }
}

impl Generic for Call {
    fn is_generic(&self) -> bool {
        self.function.is_generic() || self.args.iter().any(|arg| arg.is_generic())
    }
}
