use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::errors::Error;

/// Action to do on AST
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Throw value as an error
    Throw(Expression),
    /// Return value
    Return(Expression),
}

/// Cast expression to type
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Cast {
    /// Expression to cast
    pub expr: Expression,
    /// Type to cast to
    pub ty: Expression,
}

/// Expression that can be evaluated to a value
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Expression {
    /// A JSON value
    Value(Value),
    /// A variable reference
    #[from(ignore)]
    Variable(String),
    /// A cast from one type to another
    Cast(Box<Cast>),
}

#[derive(Serialize, Deserialize, From)]
#[serde(untagged)]
enum ExpressionDTO {
    Tagged(TaggedExpressionDTO),

    Value(Value),
}

impl From<Expression> for ExpressionDTO {
    fn from(value: Expression) -> Self {
        match value {
            Expression::Value(value) => ExpressionDTO::Value(value),
            Expression::Variable(name) => {
                ExpressionDTO::Tagged(TaggedExpressionDTO::Variable(name))
            }
            Expression::Cast(cast) => ExpressionDTO::Tagged(TaggedExpressionDTO::Cast(cast.into())),
        }
    }
}

impl From<ExpressionDTO> for Expression {
    fn from(value: ExpressionDTO) -> Self {
        match value {
            ExpressionDTO::Value(value) => Expression::Value(value),
            ExpressionDTO::Tagged(TaggedExpressionDTO::Variable(name)) => {
                Expression::Variable(name)
            }
            ExpressionDTO::Tagged(TaggedExpressionDTO::Cast(cast)) => Expression::Cast(cast.into()),
        }
    }
}

#[derive(Serialize, Deserialize, From)]
enum TaggedExpressionDTO {
    Variable(String),
    Cast(Box<CastDTO>),
}

#[derive(Serialize, Deserialize)]
struct CastDTO {
    pub expr: ExpressionDTO,
    pub ty: ExpressionDTO,
}

impl From<Box<Cast>> for Box<CastDTO> {
    fn from(value: Box<Cast>) -> Self {
        Box::new(CastDTO {
            expr: value.expr.into(),
            ty: value.ty.into(),
        })
    }
}

impl From<Box<CastDTO>> for Box<Cast> {
    fn from(value: Box<CastDTO>) -> Self {
        Box::new(Cast {
            expr: value.expr.into(),
            ty: value.ty.into(),
        })
    }
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let dto: ExpressionDTO = self.clone().into();
        dto.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto: ExpressionDTO = Deserialize::deserialize(deserializer)?;
        Ok(dto.into())
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        Expression::Value(value.into())
    }
}

impl Expression {
    /// Evaluate expression to value
    pub fn evaluate(&self, variables: &Map<String, Value>) -> Result<Value, Error> {
        match self {
            Expression::Value(value) => Ok(value.clone()),
            Expression::Variable(name) => {
                let value = variables.get(name).unwrap();
                Ok(value.clone())
            }
            Expression::Cast(cast) => {
                let value = cast.expr.evaluate(variables)?;
                let ty = cast.ty.evaluate(variables)?;
                Ok(json!({ ty.as_str().unwrap(): value }))
            }
        }
    }
}

impl Action {
    /// Execute this action with expanding variables
    pub fn execute(&self, variables: &Map<String, Value>) -> Result<Value, Error> {
        match self {
            Action::Throw(expr) => Err(serde_json::from_value(expr.evaluate(variables)?).unwrap()),
            Action::Return(expr) => Ok(expr.evaluate(variables)?),
        }
    }
}

/// A JSON value that represents a variable reference.
pub fn reference(name: &str) -> Expression {
    Expression::Variable(name.to_string())
}

/// Create a throw action
pub fn throw(expr: impl Into<Expression>) -> Action {
    Action::Throw(expr.into())
}

/// Create a return action
pub fn ret(expr: impl Into<Expression>) -> Action {
    Action::Return(expr.into())
}

/// Create a cast expression
pub fn cast(expr: impl Into<Expression>, typename: impl Into<Expression>) -> Expression {
    Expression::Cast(Box::new(Cast {
        expr: expr.into(),
        ty: typename.into(),
    }))
}
