use std::sync::Arc;
use std::borrow::Borrow;
use std::hash::Hash;

/// Trait for named objects
pub trait Named {
	/// Returns the name of the item.
	fn name(&self) -> &str;
}

impl<T: Named> Named for Arc<T> {
	/// Returns the name of the underlying item.
	fn name(&self) -> &str {
		self.as_ref().name()
	}
}

/// Helper struct to hash declarations by name
#[derive(Debug, Eq, Clone)]
pub struct HashByName<T: Named> {
	pub value: T
}

impl<T: Named> From<T> for HashByName<T> {
	fn from(value: T) -> Self {
		Self { value }
	}
}

impl<T: Named> PartialEq for HashByName<T> {
	fn eq(&self, other: &Self) -> bool {
		self.value.name() == other.value.name()
	}
}

impl<T: Named> Hash for HashByName<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.value.name().hash(state)
	}
}

impl<T: Named> Borrow<str> for HashByName<T> {
	fn borrow(&self) -> &str {
		self.value.name()
	}
}