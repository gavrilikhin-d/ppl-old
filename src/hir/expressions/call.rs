use derive_visitor::DriveMut;

use crate::hir::{Expression, Function, FunctionNamePart, Generic, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
use std::fmt::Display;
use std::ops::Range;

use crate::DataHolder;

/// AST for function call
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct Call {
    /// Range of function call
    #[drive(skip)]
    pub range: Range<usize>,

    /// Called function
    #[drive(skip)]
    pub function: Function,
    /// Generic version of called function
    #[drive(skip)]
    pub generic: Option<Function>,

    /// Arguments to the function call
    pub args: Vec<Expression>,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        let mut arg = self.args.iter();

        write!(
            f,
            "{}",
            self.function
                .read()
                .unwrap()
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
        self.function.read().unwrap().return_type.clone()
    }
}

impl Mutable for Call {
    fn is_mutable(&self) -> bool {
        self.ty().is_mutable()
    }
}

impl Generic for Call {
    fn is_generic(&self) -> bool {
        self.function.read().unwrap().is_generic() || self.args.iter().any(|arg| arg.is_generic())
    }
}
