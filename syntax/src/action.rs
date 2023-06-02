use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{
    alts,
    bootstrap::rules::{Return, Throw},
    errors::Error,
    rule_ref, seq, Rule,
};

#[cfg(test)]
use crate::{parsers::Parser, Context};
#[cfg(test)]
use pretty_assertions::assert_eq;

/// Action to do on AST
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Throw value as an error
    Throw(Expression),
    /// Return value
    Return(Expression),
}

impl Action {
    pub fn rule() -> Rule {
        Rule::new(
            "Action",
            seq!(
                "=>",
                ("value", alts!(rule_ref!(Throw), rule_ref!(Return)))
                =>
                ret(reference("value"))
            ),
        )
    }
}
#[test]
fn action() {
    let mut context = Context::default();
    let r = Action::rule();
    assert_eq!(
        r.parse("=> 1", &mut context).ast,
        json!({
            "Return": 1
        })
    );
    assert_eq!(
        r.parse("=> throw 1", &mut context).ast,
        json!({
            "Throw": 1
        })
    );
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
    /// Flatten an array of expressions
    #[from(ignore)]
    Flatten(Vec<Expression>),
    /// Merge an array of objects into one object
    #[from(ignore)]
    Merge(Box<Expression>),
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
            Expression::Flatten(values) => ExpressionDTO::Tagged(TaggedExpressionDTO::Flatten(
                values.into_iter().map(|v| v.into()).collect(),
            )),
            Expression::Merge(expr) => {
                ExpressionDTO::Tagged(TaggedExpressionDTO::Merge(expr.into()))
            }
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
            ExpressionDTO::Tagged(t) => t.into(),
        }
    }
}

impl From<TaggedExpressionDTO> for Expression {
    fn from(value: TaggedExpressionDTO) -> Self {
        match value {
            TaggedExpressionDTO::Variable(name) => Expression::Variable(name),
            TaggedExpressionDTO::Cast(cast) => Expression::Cast(cast.into()),
            TaggedExpressionDTO::Merge(expr) => Expression::Merge(expr.into()),
            TaggedExpressionDTO::Flatten(values) => {
                Expression::Flatten(values.into_iter().map(|v| v.into()).collect::<Vec<_>>())
            }
        }
    }
}

impl From<Box<Expression>> for Box<ExpressionDTO> {
    fn from(value: Box<Expression>) -> Self {
        Box::new((*value).into())
    }
}

impl From<Box<ExpressionDTO>> for Box<Expression> {
    fn from(value: Box<ExpressionDTO>) -> Self {
        Box::new((*value).into())
    }
}

#[derive(Serialize, Deserialize, From)]
enum TaggedExpressionDTO {
    Variable(String),
    Cast(Box<CastDTO>),
    Merge(Box<ExpressionDTO>),
    Flatten(Vec<ExpressionDTO>),
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
            Expression::Flatten(values) => {
                let values = values
                    .iter()
                    .map(|v| v.evaluate(variables))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut result = Vec::new();
                for value in values {
                    if let Value::Array(arr) = value {
                        for value in arr {
                            result.push(value);
                        }
                    } else {
                        result.push(value);
                    }
                }
                Ok(result.into())
            }
            Expression::Merge(expr) => {
                let mut v = expr.evaluate(variables)?;
                let objs = v.as_array_mut().unwrap();
                let mut result = Map::new();
                for obj in objs {
                    result.append(obj.as_object_mut().unwrap());
                }
                Ok(result.into())
            }
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

/// Merge an array of objects into one object
pub fn merge(expr: impl Into<Expression>) -> Expression {
    Expression::Merge(Box::new(expr.into()))
}

/// Create a cast expression
pub fn cast(expr: impl Into<Expression>, typename: impl Into<Expression>) -> Expression {
    Expression::Cast(Box::new(Cast {
        expr: expr.into(),
        ty: typename.into(),
    }))
}
