extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Identifier, Lexer, Parse, Ranged, StringWithOffset, Token,
};

use super::{parse_atomic_expression, Expression};

/// AST for member reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct MemberReference {
    /// Base expression
    pub base: Box<Expression>,
    /// Referenced member name
    pub name: Identifier,
}

impl MemberReference {
    /// Parse the rest of member references, if you have base
    pub(crate) fn parse_with_base(
        context: &mut Context<impl Lexer>,
        mut base: Box<Expression>,
    ) -> Result<Self, <Self as Parse>::Err> {
        while context.lexer.consume(Token::Dot).is_ok() {
            let name = context.consume_id()?;
            base = Box::new(MemberReference { base, name }.into());
        }
        return Ok((*base).try_into().unwrap());
    }
}

impl Parse for MemberReference {
    type Err = ParseError;

    /// Parse member reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let expr = parse_atomic_expression(context)?;

        if !matches!(expr, Expression::MemberReference(_)) {
            todo!("expected member reference error")
        }

        Ok(expr.try_into().unwrap())
    }
}

impl Ranged for MemberReference {
    fn start(&self) -> usize {
        self.base.start()
    }

    fn end(&self) -> usize {
        self.name.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::VariableReference;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_one_level_referencing() {
        let m = "point.x".parse::<MemberReference>().unwrap();
        assert_eq!(
            m,
            MemberReference {
                name: Identifier::from("x").at(6),
                base: Box::new(
                    VariableReference {
                        name: Identifier::from("point"),
                    }
                    .into()
                ),
            }
        );
    }

    #[test]
    fn test_multiple_level_referencing() {
        let m = "var.ty.name".parse::<MemberReference>().unwrap();
        assert_eq!(
            m,
            MemberReference {
                name: Identifier::from("name").at(7),
                base: Box::new(
                    MemberReference {
                        name: Identifier::from("ty").at(4),
                        base: Box::new(
                            VariableReference {
                                name: Identifier::from("var"),
                            }
                            .into()
                        ),
                    }
                    .into()
                ),
            }
        );
    }
}
