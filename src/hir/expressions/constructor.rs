use crate::hir::{Generic, Member, Type, Typed};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;
use std::fmt::Display;
use std::ops::Range;
use std::sync::Arc;

use super::{Expression, TypeReference};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Initializer {
    /// Range of name of member reference
    pub span: Range<usize>,
    /// Index of referenced member
    pub index: usize,
    /// Initialized member
    pub member: Arc<Member>,
    /// Value to initialize with
    pub value: Expression,
}

impl Display for Initializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let member = self.member.name();
        let value = self.value.to_string();
        if member != value {
            write!(f, "{}: {}", member, value)
        } else {
            write!(f, "{}", member)
        }
    }
}

impl Ranged for Initializer {
    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.value.end()
    }
}

impl Generic for Initializer {
    fn is_generic(&self) -> bool {
        self.member.is_generic() || self.value.is_generic()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Constructor {
    /// Type of constructed object
    pub ty: TypeReference,
    /// Initializers of constructed object
    pub initializers: Vec<Initializer>,
    /// Location of rbrace
    pub rbrace: usize,
}

impl Display for Constructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        write!(f, "{} {{ ", self.ty().name())?;
        write!(
            f,
            "{}",
            self.initializers
                .iter()
                .map(|initializer| initializer.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        write!(f, " }}")
    }
}

impl Mutable for Constructor {
    fn is_mutable(&self) -> bool {
        false
    }
}

impl Ranged for Constructor {
    fn start(&self) -> usize {
        self.ty.start()
    }

    fn end(&self) -> usize {
        self.rbrace + 1
    }
}

impl Typed for Constructor {
    /// Get type of variable reference
    fn ty(&self) -> Type {
        self.ty.referenced_type.clone()
    }
}

impl Generic for Constructor {
    fn is_generic(&self) -> bool {
        self.ty.is_generic() || self.initializers.iter().any(|i| i.is_generic())
    }
}
