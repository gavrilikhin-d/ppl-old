use crate::hir::{Type, Typed, Member};
use crate::mutability::Mutable;
use crate::syntax::Ranged;
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Constructor {
	/// Type of constructed object
	pub ty: TypeReference,
	/// Initializers of constructed object
	pub initializers: Vec<Initializer>,
	/// Location of rbrace
	pub rbrace: usize,
}

impl Mutable for Constructor {
	fn is_mutable(&self) -> bool { false }
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
