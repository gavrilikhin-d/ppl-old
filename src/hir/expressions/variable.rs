use derive_more::From;

use crate::hir::{Generic, Parameter, Type, Typed, VariableDeclaration};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;
use std::borrow::Cow;
use std::sync::Arc;

/// Parameter or variable declaration
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ParameterOrVariable {
    Variable(Arc<VariableDeclaration>),
    Parameter(Arc<Parameter>),
}

impl From<Parameter> for ParameterOrVariable {
    fn from(parameter: Parameter) -> Self {
        Self::Parameter(Arc::new(parameter))
    }
}

impl From<VariableDeclaration> for ParameterOrVariable {
    fn from(variable: VariableDeclaration) -> Self {
        Self::Variable(Arc::new(variable))
    }
}

impl Named for ParameterOrVariable {
    fn name(&self) -> Cow<'_, str> {
        match self {
            ParameterOrVariable::Variable(variable) => variable.name(),
            ParameterOrVariable::Parameter(parameter) => parameter.name(),
        }
    }
}

impl Typed for ParameterOrVariable {
    fn ty(&self) -> Type {
        match self {
            ParameterOrVariable::Variable(variable) => variable.ty(),
            ParameterOrVariable::Parameter(parameter) => parameter.ty(),
        }
    }
}

impl Mutable for ParameterOrVariable {
    fn is_mutable(&self) -> bool {
        match self {
            ParameterOrVariable::Variable(variable) => variable.is_mutable(),
            ParameterOrVariable::Parameter(parameter) => parameter.is_mutable(),
        }
    }
}

impl Generic for ParameterOrVariable {
    fn is_generic(&self) -> bool {
        match self {
            ParameterOrVariable::Variable(variable) => variable.is_generic(),
            ParameterOrVariable::Parameter(parameter) => parameter.is_generic(),
        }
    }
}

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableReference {
    /// Range of variable reference
    pub span: std::ops::Range<usize>,
    /// Referenced variable name
    pub variable: ParameterOrVariable,
}

impl Mutable for VariableReference {
    /// Check if referenced variable is mutable
    fn is_mutable(&self) -> bool {
        self.variable.is_mutable()
    }
}

impl Ranged for VariableReference {
    /// Get range of variable reference
    fn range(&self) -> std::ops::Range<usize> {
        self.span.clone()
    }
}

impl Typed for VariableReference {
    /// Get type of variable reference
    fn ty(&self) -> Type {
        self.variable.ty()
    }
}

impl Generic for VariableReference {
    fn is_generic(&self) -> bool {
        self.variable.is_generic()
    }
}
