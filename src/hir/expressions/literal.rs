use std::fmt::Display;

use derive_visitor::DriveMut;
use runtime::maybe_to_decimal_string;

use crate::hir::{Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub enum Literal {
    /// None literal
    #[drive(skip)]
    None { offset: usize, ty: Type },
    /// Bool literal
    #[drive(skip)]
    Bool {
        offset: usize,
        value: bool,
        ty: Type,
    },
    /// Any precision decimal integer literal
    #[drive(skip)]
    Integer {
        span: std::ops::Range<usize>,
        value: rug::Integer,
        ty: Type,
    },
    /// Any precision decimal rational literal
    #[drive(skip)]
    Rational {
        span: std::ops::Range<usize>,
        value: rug::Rational,
        ty: Type,
    },
    /// String literal
    #[drive(skip)]
    String {
        span: std::ops::Range<usize>,
        value: String,
        ty: Type,
    },
}

impl Display for Literal {
    /// Display literal
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::None { .. } => write!(f, "none"),
            Literal::Bool { value, .. } => write!(f, "{}", value),
            Literal::Integer { value, .. } => write!(f, "{}", value),
            Literal::Rational { value, .. } => write!(f, "{}", maybe_to_decimal_string(value)),
            Literal::String { value, .. } => write!(f, "{:?}", value),
        }
    }
}

impl Ranged for Literal {
    /// Get range of literal
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Literal::None { offset, .. } => *offset..*offset + "none".len(),
            Literal::Bool { offset, value, .. } => *offset..*offset + format!("{}", value).len(),
            Literal::Integer { span, .. } => span.clone(),
            Literal::Rational { span, .. } => span.clone(),
            Literal::String { span, .. } => span.clone(),
        }
    }
}

impl Typed for Literal {
    /// Get type of literal
    fn ty(&self) -> Type {
        match self {
            Literal::None { ty, .. } => ty,
            Literal::Bool { ty, .. } => ty,
            Literal::Integer { ty, .. } => ty,
            Literal::Rational { ty, .. } => ty,
            Literal::String { ty, .. } => ty,
        }
        .clone()
    }
}

impl Mutable for Literal {
    /// Literal is always immutable
    fn is_immutable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compilation::Compiler,
        hir::ModuleData,
        semantics::{Context, ModuleContext},
    };

    use super::*;

    #[test]
    fn test_literal_display() {
        let mut compiler = Compiler::new();
        let context = ModuleContext::new(ModuleData::default(), &mut compiler);
        let literal_none = Literal::None {
            offset: 0,
            ty: context.builtin().types().none(),
        };
        assert_eq!(format!("{}", literal_none), "none");

        let literal_bool = Literal::Bool {
            offset: 0,
            value: true,
            ty: context.builtin().types().bool(),
        };
        assert_eq!(format!("{}", literal_bool), "true");

        let literal_bool = Literal::Bool {
            offset: 0,
            value: false,
            ty: context.builtin().types().bool(),
        };
        assert_eq!(format!("{}", literal_bool), "false");

        let literal_integer = Literal::Integer {
            span: 0..1,
            value: rug::Integer::from(42),
            ty: context.builtin().types().integer(),
        };
        assert_eq!(format!("{}", literal_integer), "42");

        let literal_rational = Literal::Rational {
            span: 0..1,
            value: rug::Rational::from_f32(0.5).unwrap(),
            ty: context.builtin().types().rational(),
        };
        assert_eq!(format!("{}", literal_rational), "0.5");

        let literal_rational = Literal::Rational {
            span: 0..1,
            value: rug::Rational::from((1, 3)),
            ty: context.builtin().types().rational(),
        };
        assert_eq!(format!("{}", literal_rational), "1/3");

        let literal_string = Literal::String {
            span: 0..1,
            value: String::from("hello"),
            ty: context.builtin().types().string(),
        };
        assert_eq!(format!("{}", literal_string), r#""hello""#);
    }
}
