use crate::hir::{Generic, Member, Type, Typed};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;
use std::fmt::Display;
use std::sync::Arc;

use super::Expression;

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MemberReference {
    /// Range of name of member reference
    pub span: std::ops::Range<usize>,
    /// Base expression
    pub base: Box<Expression>,
    /// Referenced variable name
    pub member: Arc<Member>,
    /// Index of referenced member
    pub index: usize,
}

impl Display for MemberReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = &self.base;
        let name = self.member.name();
        write!(f, "{base}.{name}")
    }
}

impl Mutable for MemberReference {
    /// Check if referenced variable is mutable
    fn is_mutable(&self) -> bool {
        self.base.is_mutable()
    }
}

impl Ranged for MemberReference {
    fn start(&self) -> usize {
        self.base.start()
    }

    fn end(&self) -> usize {
        self.span.end
    }
}

impl Typed for MemberReference {
    /// Get type of variable reference
    fn ty(&self) -> Type {
        self.member.ty()
    }
}

impl Generic for MemberReference {
    fn is_generic(&self) -> bool {
        self.base.is_generic() || self.member.is_generic()
    }
}
