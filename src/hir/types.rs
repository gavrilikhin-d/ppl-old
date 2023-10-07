use std::{
    fmt::Display,
    sync::{Arc, Weak},
};

use crate::{mutability::Mutable, named::Named, syntax::StringWithOffset};

use super::{Generic, Member, Module, Specialized, TraitDeclaration, TypeDeclaration};
use derive_more::{From, TryInto};
use enum_dispatch::enum_dispatch;

use super::Expression;

/// PPL's Function type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionType {
    /// Parameters
    pub parameters: Vec<Type>,
    /// Return type
    pub return_type: Box<Type>,
    /// Cached name of function type
    name: String,
}

impl FunctionType {
    /// Build new function type
    pub fn build() -> FunctionTypeBuilder {
        FunctionTypeBuilder::new()
    }
}

impl Named for FunctionType {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for FunctionType
pub struct FunctionTypeBuilder {
    /// Parameters
    pub parameters: Vec<Type>,
}

impl FunctionTypeBuilder {
    /// Create new builder for function type
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }

    /// Set parameter to function type
    pub fn with_parameters(mut self, parameters: Vec<Type>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set return type to function type and build function
    pub fn with_return_type(self, return_type: Type) -> FunctionType {
        let name = self.build_name(&return_type);
        FunctionType {
            parameters: self.parameters,
            return_type: Box::new(return_type),
            name,
        }
    }

    /// Build name of function type
    fn build_name(&self, return_type: &Type) -> String {
        let mut name = String::new();
        name.push_str("(");
        for (i, parameter) in self.parameters.iter().enumerate() {
            if i != 0 {
                name.push_str(", ");
            }
            name.push_str(parameter.name());
        }
        name.push_str(&format!(") -> {}", return_type.name()));
        name
    }
}

/// Self type is used to represent type of self in trait methods
#[derive(Debug, Clone)]
pub struct SelfType {
    /// Trait associated with self type
    pub associated_trait: Weak<TraitDeclaration>,
}

impl PartialEq for SelfType {
    fn eq(&self, other: &Self) -> bool {
        self.associated_trait.ptr_eq(&other.associated_trait)
    }
}
impl Eq for SelfType {}

impl Named for SelfType {
    fn name(&self) -> &str {
        "Self"
    }
}

/// Type of a generic parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericType {
    /// Name of the generic type
    pub name: StringWithOffset,
}

impl Named for GenericType {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Specialized type
pub type SpecializedType = Specialized<Type>;

impl Named for SpecializedType {
    fn name(&self) -> &str {
        self.specialized.name()
    }
}

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Type {
    /// User defined type
    Class(Arc<TypeDeclaration>),
    /// User defined trait
    Trait(Arc<TraitDeclaration>),
    /// Self type and trait it represents
    SelfType(SelfType),
    /// Type for generic parameters
    Generic(GenericType),
    /// Specialized type
    Specialized(Box<SpecializedType>),
    /// Function type
    Function(FunctionType),
}

impl Type {
    /// Return most specialized subtype
    pub fn specialized(&self) -> Type {
        match self {
            Type::Specialized(s) => s.specialized.specialized(),
            _ => self.clone(),
        }
    }

    /// Get members of type
    pub fn members(&self) -> &[Arc<Member>] {
        match self {
            Type::Class(c) => c.members(),
            _ => &[],
        }
    }

    /// Map self type to given type
    pub fn map_self<'s>(&'s self, ty: &'s Type) -> &'s Type {
        match self {
            Type::SelfType(_) => ty,
            _ => self,
        }
    }

    /// Is this a builtin type?
    pub fn is_builtin(&self) -> bool {
        match self {
            Type::Class(c) => c.is_builtin(),
            _ => false,
        }
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        match self {
            Type::Class(c) => c.is_none(),
            _ => false,
        }
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        match self {
            Type::Class(c) => c.is_bool(),
            _ => false,
        }
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        match self {
            Type::Class(c) => c.is_integer(),
            _ => false,
        }
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        match self {
            Type::Class(c) => c.is_string(),
            _ => false,
        }
    }

    /// Get builtin type by name
    fn get_builtin(name: &str) -> Type {
        Module::builtin().types.get(name).unwrap().clone().into()
    }

    /// Get builtin "None" type
    pub fn none() -> Type {
        Type::get_builtin("None")
    }

    /// Get builtin "Bool" type
    pub fn bool() -> Type {
        Type::get_builtin("Bool")
    }

    /// Get builtin "Integer" type
    pub fn integer() -> Type {
        Type::get_builtin("Integer")
    }

    /// Get builtin "String" type
    pub fn string() -> Type {
        Type::get_builtin("String")
    }
}

impl Generic for Type {
    fn is_generic(&self) -> bool {
        match self {
            Type::SelfType(_) | Type::Trait(_) | Type::Generic(_) => true,
            Type::Specialized(s) => s.specialized.is_generic(),
            Type::Class(c) => c.members.iter().any(|m| m.ty().is_generic()),
            Type::Function(f) => {
                f.parameters.iter().any(|p| p.is_generic()) || f.return_type.is_generic()
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Named for Type {
    fn name(&self) -> &str {
        match self {
            Type::Class(class) => class.name(),
            Type::Trait(tr) => tr.name(),
            Type::SelfType(s) => s.name(),
            Type::Function(f) => f.name(),
            Type::Generic(g) => g.name(),
            Type::Specialized(s) => s.name(),
        }
    }
}

impl Mutable for Type {
    fn is_mutable(&self) -> bool {
        false
    }
}

/// Trait for values with a type
#[enum_dispatch]
pub trait Typed {
    /// Get type of value
    fn ty(&self) -> Type;
}
