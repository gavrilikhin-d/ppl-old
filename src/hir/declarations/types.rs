use std::sync::Arc;

use crate::{
    hir::{GenericType, Type, Typed},
    named::Named,
    syntax::StringWithOffset,
};

/// Member of type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member {
    /// Member's name
    pub name: StringWithOffset,
    /// Member's type
    pub ty: Type,
}

impl Named for Member {
    /// Get name of member
    fn name(&self) -> &str {
        &self.name
    }
}

impl Typed for Member {
    /// Get type of member
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeDeclaration {
    /// Type's name
    pub name: StringWithOffset,
    /// Generic parameters of type
    pub generic_parameters: Vec<GenericType>,
    /// Is this type from builtin module?
    pub is_builtin: bool,
    /// Members of type
    pub members: Vec<Arc<Member>>,
}

impl TypeDeclaration {
    /// Get member by name
    pub fn members(&self) -> &[Arc<Member>] {
        self.members.as_slice()
    }

    /// Is this a builtin type?
    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        self.is_builtin && self.name == "None"
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        self.is_builtin && self.name == "Bool"
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        self.is_builtin && self.name == "Integer"
    }

    /// Is this a builtin "Rational" type?
    pub fn is_rational(&self) -> bool {
        self.is_builtin && self.name == "Rational"
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        self.is_builtin && self.name == "String"
    }

    /// Is this an opaque type?
    pub fn is_opaque(&self) -> bool {
        self.members.is_empty()
    }

    /// Is this a generic type?
    ///
    /// Note: even if [`generic_parameters`](Self::generic_parameters)
    /// aren't empty, type may be non-generic, if all members are non-generic
    pub fn is_generic(&self) -> bool {
        !self.generic_parameters.is_empty() && self.members.iter().any(|m| m.ty().is_generic())
    }
}

impl Named for TypeDeclaration {
    /// Get name of type
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;
    use crate::hir::{Member, Type};
    use crate::semantics::ASTLowering;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_type_without_body() {
        let type_decl = "type x"
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .lower_to_hir()
            .unwrap();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                name: StringWithOffset::from("x").at(5),
                generic_parameters: vec![],
                is_builtin: false,
                members: vec![],
            }
        );
    }

    #[test]
    fn type_with_generics() {
        let type_decl = "type Point<U>:\n\tx: U"
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .lower_to_hir()
            .unwrap();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                name: StringWithOffset::from("Point").at(5),
                generic_parameters: vec![GenericType {
                    name: StringWithOffset::from("U").at(11),
                }],
                is_builtin: false,
                members: vec![Arc::new(Member {
                    name: StringWithOffset::from("x").at(16),
                    ty: GenericType {
                        name: StringWithOffset::from("U").at(11),
                    }
                    .into(),
                }),],
            }
        );
    }

    #[test]
    fn test_type_with_body() {
        let type_decl = include_str!("../../../examples/point.ppl")
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .lower_to_hir()
            .unwrap();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                name: StringWithOffset::from("Point").at(5),
                generic_parameters: vec![],
                is_builtin: false,
                members: vec![
                    Arc::new(Member {
                        name: StringWithOffset::from("x").at(13),
                        ty: Type::integer(),
                    }),
                    Arc::new(Member {
                        name: StringWithOffset::from("y").at(16),
                        ty: Type::integer(),
                    }),
                ],
            }
        );
    }
}
