use std::borrow::Cow;
use std::fmt::Display;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

use derive_visitor::DriveMut;

use crate::hir::{Expression, Generic, Type, TypeReference, Typed};
use crate::mutability::{Mutability, Mutable};
use crate::named::Named;
use crate::syntax::{Identifier, Keyword, Ranged};

/// Variable data holder
#[derive(Debug, Clone)]
pub struct Variable {
    inner: Arc<RwLock<VariableData>>,
}

impl Variable {
    /// Create a new variable from its data
    pub fn new(data: VariableData) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Lock variable for reading
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, VariableData>> {
        self.inner.read()
    }

    /// Lock variable for writing
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, VariableData>> {
        self.inner.write()
    }

    /// Is this a temporary variable?
    pub fn is_temporary(&self) -> bool {
        self.read().unwrap().is_temporary()
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().unwrap().fmt(f)
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        *self.read().unwrap() == *other.read().unwrap()
    }
}

impl Eq for Variable {}

impl Named for Variable {
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Generic for Variable {
    fn is_generic(&self) -> bool {
        self.read().unwrap().is_generic()
    }
}

impl Typed for Variable {
    fn ty(&self) -> Type {
        self.read().unwrap().ty()
    }
}

impl Mutable for Variable {
    fn is_mutable(&self) -> bool {
        self.read().unwrap().is_mutable()
    }
}

impl Ranged for Variable {
    fn range(&self) -> std::ops::Range<usize> {
        self.read().unwrap().range()
    }
}

impl DriveMut for Variable {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        self.write().unwrap().drive_mut(visitor)
    }
}

/// Declaration of a variable
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct VariableData {
    /// Keyword `let`
    #[drive(skip)]
    pub keyword: Keyword<"let">,
    /// Mutability of variable
    #[drive(skip)]
    pub mutability: Mutability,
    /// Variable's name
    #[drive(skip)]
    pub name: Identifier,
    /// Type reference for variable
    #[drive(skip)]
    pub type_reference: Option<TypeReference>,
    /// Type of variable
    #[drive(skip)]
    pub ty: Type,
    /// Initializer for variable
    pub initializer: Option<Expression>,
}

impl VariableData {
    /// Is this a temporary variable?
    pub fn is_temporary(&self) -> bool {
        self.name.starts_with("$tmp")
    }
}

impl Display for VariableData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;
        write!(
            f,
            "let {}{}: {}{}",
            if self.mutability == Mutability::Mutable {
                "mut "
            } else {
                ""
            },
            self.name,
            self.ty(),
            self.initializer
                .as_ref()
                .map_or("".to_string(), |i| format!(" = {i}"))
        )
    }
}

impl Named for VariableData {
    /// Get name of variable
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Mutable for VariableData {
    /// Is variable declared as mutable?
    fn is_mutable(&self) -> bool {
        self.mutability.is_mutable()
    }
}

impl Typed for VariableData {
    /// Get type of variable
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}

impl Generic for VariableData {
    fn is_generic(&self) -> bool {
        self.ty.is_generic()
    }
}

impl Ranged for VariableData {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.initializer
            .as_ref()
            .map_or(self.name.end(), |i| i.end())
    }
}
