mod call;
pub use call::*;

mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod tuple;
pub use tuple::*;

mod r#type;
pub use r#type::*;

mod member;
pub use member::*;

mod constructor;
pub use constructor::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingExpression, ParseError},
    Context, Lexer, OperatorKind, Parse, Ranged, StartsHere, Token,
};

use derive_more::{From, TryInto};

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From, TryInto)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    Call(Call),
    Tuple(Tuple),
    TypeReference(TypeReference),
    MemberReference(MemberReference),
    Constructor(Constructor),
}

impl StartsHere for Expression {
    /// Check that expression 100% starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        Literal::starts_here(context)
            || VariableReference::starts_here(context)
            || TypeReference::starts_here(context)
            || Tuple::starts_here(context)
            || matches!(
                context.lexer.peek(),
                Some(Token::Operator(_) | Token::Less | Token::Greater)
            )
    }
}

/// Parse atomic expression
fn parse_atomic_expression(context: &mut Context<impl Lexer>) -> Result<Expression, ParseError> {
    let expr: Expression = if Literal::starts_here(context) {
        Literal::parse(context)?.into()
    } else if Tuple::starts_here(context) {
        Tuple::parse(context)?.into()
    } else if VariableReference::starts_here(context) {
        VariableReference::parse(context)?.into()
    } else if TypeReference::starts_here(context) {
        let ty = TypeReference::parse(context)?;
        if context.lexer.try_match(Token::LBrace).is_err() {
            ty.into()
        } else {
            Constructor::parse_with_ty(context, ty)?.into()
        }
    } else {
        return Err(MissingExpression {
            at: context.lexer.span().end.into(),
        }
        .into());
    };

    if context.lexer.try_match(Token::Dot).is_ok() {
        return Ok(MemberReference::parse_with_base(context, Box::new(expr))?.into());
    }

    Ok(expr)
}

/// postfix-expression: atomic-expression postfix-operator?
fn parse_postfix_expression(context: &mut Context<impl Lexer>) -> Result<Expression, ParseError> {
    let operand = parse_atomic_expression(context)?;

    Ok(
        if let Ok(operator) = context
            .lexer
            .consume(Token::Operator(OperatorKind::Postfix))
        {
            Call {
                kind: FnKind::Operator,
                name_parts: vec![operand.into(), operator.into()],
            }
            .into()
        } else {
            operand
        },
    )
}

/// prefix-expression: prefix-operator? postfix-expression
fn parse_prefix_expression(context: &mut Context<impl Lexer>) -> Result<Expression, ParseError> {
    let operator = context.lexer.consume(Token::Operator(OperatorKind::Prefix));
    let operand = parse_postfix_expression(context)?;

    Ok(if let Ok(operator) = operator {
        Call {
            kind: FnKind::Operator,
            name_parts: vec![operator.into(), operand.into()],
        }
        .into()
    } else {
        operand
    })
}

/// Parse right hand side of binary expression
fn parse_binary_rhs(
    context: &mut Context<impl Lexer>,
    prev_op: Option<&str>,
    mut left: Expression,
) -> Result<Expression, ParseError> {
    while context.lexer.peek().is_some_and(|t| t.is_infix_operator()) {
        let op = context.lexer.consume_operator()?;

        if prev_op
            .is_some_and(|prev_op| context.precedence_groups.has_less_precedence(&op, prev_op))
        {
            break;
        }

        let mut right = parse_prefix_expression(context)?;
        if context.lexer.peek().is_some_and(|t| t.is_infix_operator()) {
            let next_op = context.lexer.peek_slice();
            if context
                .precedence_groups
                .has_greater_precedence(next_op, &op)
            {
                right = parse_binary_rhs(context, Some(&op), right)?;
            }
        }

        left = Call {
            kind: FnKind::Operator,
            name_parts: vec![left.into(), op.into(), right.into()],
        }
        .into();
    }

    Ok(left)
}

/// Parse binary expression
pub(crate) fn parse_binary_expression(
    context: &mut Context<impl Lexer>,
) -> Result<Expression, ParseError> {
    let left = parse_prefix_expression(context)?;
    parse_binary_rhs(context, None, left)
}

impl Parse for Expression {
    type Err = ParseError;

    /// Parse expression using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if !Expression::starts_here(context) {
            return Err(MissingExpression {
                at: context.lexer.span().end.into(),
            }
            .into());
        }

        let call = Call::parse(context)?;
        if call.name_parts.len() > 1 {
            return Ok(call.into());
        }

        Ok(match call.name_parts.first().unwrap() {
            CallNamePart::Argument(arg) => arg.clone(),
            CallNamePart::Text(t) => {
                if t.as_str().chars().nth(0).unwrap().is_uppercase() {
                    TypeReference {
                        generic_parameters: vec![],
                        name: t.clone(),
                    }
                    .into()
                } else {
                    VariableReference { name: t.clone() }.into()
                }
            }
        })
    }
}

impl Ranged for Expression {
    /// Get range of expression
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Expression::Literal(l) => l.range(),
            Expression::VariableReference(var) => var.range(),
            Expression::Call(call) => call.range(),
            Expression::Tuple(tuple) => tuple.range(),
            Expression::TypeReference(ty_ref) => ty_ref.range(),
            Expression::MemberReference(m) => m.range(),
            Expression::Constructor(c) => c.range(),
        }
    }
}
