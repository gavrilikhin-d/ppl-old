use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    sync::{Arc, Weak},
};

use crate::{mutability::Mutable, named::Named, syntax::StringWithOffset, AddSourceLocation};

use super::{
    Generic, GenericName, Member, Specialize, Specialized, TraitDeclaration, TypeDeclaration,
    TypeReference,
};
use derive_more::{Display, From, TryInto};
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
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Generic for FunctionType {
    fn is_generic(&self) -> bool {
        self.parameters.iter().any(|p| p.is_generic()) || self.return_type.is_generic()
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
            name.push_str(&parameter.generic_name());
        }
        name.push_str(&format!(") -> {}", return_type.generic_name()));
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
    fn name(&self) -> Cow<'_, str> {
        "Self".into()
    }
}

impl Display for SelfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Type of a generic parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericType {
    /// Name of the generic type
    pub name: StringWithOffset,
    /// Constraint for this type
    pub constraint: Option<TypeReference>,
}

impl Named for GenericType {
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for GenericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Specialized type
pub type SpecializedType = Specialized<Type>;

impl Named for SpecializedType {
    fn name(&self) -> Cow<'_, str> {
        self.specialized.name()
    }
}

impl GenericName for SpecializedType {
    fn generic_name(&self) -> Cow<'_, str> {
        self.specialized.generic_name()
    }
}

impl Display for SpecializedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.specialized)
    }
}

impl Debug for SpecializedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} => {})", self.generic, self.specialized)
    }
}

/// Type of values
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto)]
pub enum Type {
    /// User defined type
    Class(Arc<TypeDeclaration>),
    /// User defined trait
    Trait(Arc<TraitDeclaration>),
    /// Self type and trait it represents
    SelfType(SelfType),
    /// Type for generic parameters
    Generic(Box<GenericType>),
    /// Specialized type
    Specialized(Box<SpecializedType>),
    /// Function type
    Function(FunctionType),
}

impl From<SpecializedType> for Type {
    fn from(specialized: SpecializedType) -> Self {
        Self::Specialized(Box::new(specialized))
    }
}

impl From<GenericType> for Type {
    fn from(generic: GenericType) -> Self {
        Box::new(generic).into()
    }
}

impl Type {
    /// Get diff in specializations
    /// that needed to make `self` type to match `target` type.
    ///
    /// # Note
    /// This function ignores the fact that types may be non-generic
    pub fn diff(&self, target: Type) -> Vec<SpecializedType> {
        let from = self.specialized();
        let to = target.specialized();
        if from == to {
            return vec![];
        }

        match (&from, &to) {
            (Type::Class(from), Type::Class(to)) if from.name == to.name => from
                .generic_parameters
                .iter()
                .zip(to.generic_parameters.iter())
                .flat_map(|(t1, t2)| t1.diff(t2.clone()))
                .collect(),
            _ => vec![from.specialize_with(to).try_into().unwrap()],
        }
    }

    /// Return most specialized subtype
    pub fn specialized(&self) -> Type {
        match self {
            Type::Specialized(s) => s.specialized.specialized(),
            _ => self.clone(),
        }
    }

    /// Get this type without reference
    pub fn without_ref(&self) -> Type {
        if !self.is_reference() {
            return self.clone();
        }

        self.generics()[0].specialized()
    }

    /// Get generic parameters of type
    pub fn generics(&self) -> &[Type] {
        match self {
            Type::Class(c) => c.generics(),
            Type::Specialized(s) => &s.specialized.generics(),
            _ => &[],
        }
    }

    /// Get members of type
    pub fn members(&self) -> &[Arc<Member>] {
        match self {
            Type::Class(c) => c.members(),
            Type::Specialized(s) => &s.specialized.members(),
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
        match self.without_ref() {
            Type::Class(c) => c.is_builtin(),
            _ => false,
        }
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_none(),
            _ => false,
        }
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_bool(),
            _ => false,
        }
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_integer(),
            _ => false,
        }
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_string(),
            _ => false,
        }
    }

    /// Is this a builtin `Reference` type?
    pub fn is_reference(&self) -> bool {
        match self.specialized() {
            Type::Class(c) => c.is_reference(),
            _ => false,
        }
    }

    /// Convert this to class type
    /// # Panics
    /// Panics if this is not a class type
    pub fn as_class(self) -> Arc<TypeDeclaration> {
        self.try_into().unwrap()
    }

    /// Size of type in bytes
    pub fn size_in_bytes(&self) -> usize {
        match self.specialized() {
            Type::Class(c) => c.size_in_bytes(),
            // TODO: implement size for other types
            _ => 0,
        }
    }
}

impl Generic for Type {
    fn is_generic(&self) -> bool {
        match self {
            Type::SelfType(_) | Type::Trait(_) | Type::Generic(_) => true,
            Type::Specialized(s) => s.is_generic(),
            Type::Class(c) => c.is_generic(),
            Type::Function(f) => f.is_generic(),
        }
    }
}

impl GenericName for Type {
    fn generic_name(&self) -> Cow<'_, str> {
        match self {
            Type::Class(c) => c.generic_name(),
            Type::Specialized(s) => s.generic_name(),
            _ => self.name(),
        }
    }
}

impl Specialize<Type> for Type {
    type Specialized = SpecializedType;

    fn specialize_with(self, specialized: Type) -> SpecializedType {
        debug_assert!(self.is_generic());

        SpecializedType {
            generic: self,
            specialized,
        }
    }
}

impl Named for Type {
    fn name(&self) -> Cow<'_, str> {
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

impl AddSourceLocation for Type {}

/// Trait for values with a type
#[enum_dispatch]
pub trait Typed {
    /// Get type of value
    fn ty(&self) -> Type;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pretty_assertions::{assert_eq, assert_str_eq};

    use crate::{
        ast,
        hir::{GenericName, GenericType, Specialize, SpecializeClass, Type, TypeDeclaration},
        semantics::ASTLowering,
        syntax::StringWithOffset,
    };

    /// Get type declaration from source
    fn type_decl(source: &str) -> Arc<TypeDeclaration> {
        source
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .lower_to_hir()
            .unwrap()
    }

    #[test]
    fn generic_name() {
        let a = type_decl("type A<T, U>");
        assert_str_eq!(a.generic_name(), "A<T, U>");

        let b = type_decl("type B<T>");
        assert_str_eq!(b.generic_name(), "B<T>");

        let t: Type = GenericType {
            name: StringWithOffset::from("T").at(7),
            constraint: None,
        }
        .into();
        let u: Type = GenericType {
            name: StringWithOffset::from("U").at(10),
            constraint: None,
        }
        .into();

        let x: Type = type_decl("type X").into();
        assert_str_eq!(x.generic_name(), "X");
        let y: Type = type_decl("type Y").into();
        assert_str_eq!(y.generic_name(), "Y");

        // B<Y>
        let by: Type = b
            .specialize_with(SpecializeClass::without_members(vec![t
                .clone()
                .specialize_with(y)
                .into()]))
            .into();
        assert_str_eq!(by.generic_name(), "B<Y>");

        // A<X, B<Y>>
        let t1 = a.specialize_with(SpecializeClass::without_members(vec![
            t.specialize_with(x).into(),
            u.specialize_with(by).into(),
        ]));
        assert_str_eq!(t1.generic_name(), "A<X, B<Y>>");
    }

    #[test]
    fn diff() {
        let a = type_decl("type A<T, U>");
        let b = type_decl("type B<T>");
        let c = type_decl("type C");

        let x: Type = GenericType {
            name: "X".into(),
            constraint: None,
        }
        .into();
        let y: Type = GenericType {
            name: "Y".into(),
            constraint: None,
        }
        .into();

        let t: Type = GenericType {
            name: StringWithOffset::from("T").at(7),
            constraint: None,
        }
        .into();
        let u: Type = GenericType {
            name: StringWithOffset::from("U").at(10),
            constraint: None,
        }
        .into();

        let by: Type = b
            .clone()
            .specialize_with(SpecializeClass::without_members(vec![t
                .clone()
                .specialize_with(y.clone())
                .into()]))
            .into();
        println!("{}", by.generic_name());

        // A<X, B<Y>>
        let t1: Type = a
            .clone()
            .specialize_with(SpecializeClass::without_members(vec![
                t.clone().specialize_with(x.clone()).into(),
                u.clone().specialize_with(by.clone()).into(),
            ]))
            .into();
        println!("{}", t1.generic_name());

        // B<C>
        let bc: Type = b
            .specialize_with(SpecializeClass::without_members(vec![t
                .clone()
                .specialize_with(c.clone().into())
                .into()]))
            .into();
        // A<B<C>, B<C>>
        let t2: Type = a
            .specialize_with(SpecializeClass::without_members(vec![
                t.specialize_with(bc.clone()).into(),
                u.specialize_with(bc.clone()).into(),
            ]))
            .into();
        let diff = t1.diff(t2);
        diff.iter().for_each(|s| println!("{s}"));
        assert_eq!(
            diff,
            vec![x.specialize_with(bc), y.specialize_with(c.into())]
        );
    }
}
