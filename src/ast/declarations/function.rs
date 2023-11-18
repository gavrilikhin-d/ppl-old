extern crate ast_derive;
use ast_derive::AST;

use derive_more::From;

use crate::{
    ast::{Annotation, Expression, Statement, TypeReference},
    syntax::{
        error::ParseError, Context, Lexer, OperatorKind, Parse, Ranged, StartsHere,
        StringWithOffset, Token,
    },
};

use super::GenericParameter;

/// Parameter of function
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Parameter {
    /// Location of '<'
    pub less: usize,
    /// Parameter's name
    pub name: StringWithOffset,
    /// Parameter's type
    pub ty: TypeReference,
    /// Location of '>'
    pub greater: usize,
}

impl Ranged for Parameter {
    fn start(&self) -> usize {
        self.less
    }

    fn end(&self) -> usize {
        self.greater + 1
    }
}

impl Parse for Parameter {
    type Err = ParseError;

    /// Parse parameter using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let less = context.lexer.consume(Token::Less)?.start();

        let name = context
            .consume_id()
            .ok()
            .unwrap_or_else(|| StringWithOffset::from("").at(context.lexer.span().end));

        context.lexer.consume(Token::Colon)?;

        let ty = TypeReference::parse(context)?;

        let greater = context.lexer.consume_greater()?.start();

        Ok(Parameter {
            less,
            name,
            ty,
            greater,
        })
    }
}

/// Cell of function
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum FunctionNamePart {
    Text(StringWithOffset),
    Parameter(Parameter),
}

impl Ranged for FunctionNamePart {
    /// Get range of function name part
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            FunctionNamePart::Text(s) => s.range(),
            FunctionNamePart::Parameter(p) => p.range(),
        }
    }
}

impl Parse for FunctionNamePart {
    type Err = ParseError;

    /// Parse function name part using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let token = context.lexer.consume_one_of(&[
            Token::Id,
            Token::EscapedId,
            Token::Less,
            Token::Greater,
            Token::Operator(OperatorKind::Prefix),
            Token::Operator(OperatorKind::Infix),
            Token::Operator(OperatorKind::Postfix),
        ])?;
        match token {
            Token::Id | Token::EscapedId | Token::Greater | Token::Operator(_) => {
                Ok(context.lexer.string_with_offset().into())
            }
            Token::Less => {
                // '<' here is an operator
                if context.lexer.peek() == Some(Token::Less) {
                    return Ok(context.lexer.string_with_offset().into());
                }

                let less = context.lexer.span().start;

                let name = context
                    .consume_id()
                    .ok()
                    .unwrap_or_else(|| StringWithOffset::from("").at(context.lexer.span().end));

                context.lexer.consume(Token::Colon)?;

                let ty = TypeReference::parse(context)?;

                let greater = context.lexer.consume_greater()?.start();

                Ok(Parameter {
                    less,
                    name,
                    ty,
                    greater,
                }
                .into())
            }
            _ => unreachable!("consume_one_of returned unexpected token"),
        }
    }
}

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct FunctionDeclaration {
    /// Generic parameters of a function
    pub generic_parameters: Vec<GenericParameter>,
    /// Name parts of function
    pub name_parts: Vec<FunctionNamePart>,
    /// Return type of function
    pub return_type: Option<TypeReference>,
    /// Body of function
    pub body: Vec<Statement>,

    /// Does this function use implicit return (=>)
    pub implicit_return: bool,

    /// Annotations for function
    pub annotations: Vec<Annotation>,
}

impl StartsHere for FunctionDeclaration {
    /// Check that function declaration may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.try_match(Token::Fn).is_ok()
    }
}

impl Parse for FunctionDeclaration {
    type Err = ParseError;

    /// Parse function declaration using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let fn_token = context.lexer.consume(Token::Fn)?;

        let mut generic_parameters = Vec::new();
        if context
            .lexer
            .try_match(Token::Less)
            .is_ok_and(|t| t.start() == fn_token.end())
        {
            context.lexer.consume(Token::Less).unwrap();
            generic_parameters = context.parse_comma_separated(GenericParameter::parse);
            context.lexer.consume_greater()?;
        }

        let mut name_parts = Vec::new();

        loop {
            let part = FunctionNamePart::parse(context)?;
            name_parts.push(part);

            match context.lexer.peek() {
                None
                | Some(Token::Arrow)
                | Some(Token::FatArrow)
                | Some(Token::Newline)
                | Some(Token::Colon) => break,
                _ => continue,
            }
        }

        let return_type = if context.lexer.consume(Token::Arrow).is_ok() {
            Some(TypeReference::parse(context)?)
        } else {
            None
        };

        let mut body = Vec::new();
        let mut implicit_return = false;
        if context.lexer.consume(Token::FatArrow).is_ok() {
            body.push(Expression::parse(context)?.into());
            implicit_return = true;

            context.consume_eol()?;
        } else if context.lexer.consume(Token::Colon).is_ok() {
            body = context.parse_block(Statement::parse)?;
        } else {
            context.consume_eol()?;
        }

        Ok(FunctionDeclaration {
            generic_parameters,
            name_parts,
            return_type,
            body,
            implicit_return,
            annotations: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            FunctionDeclaration, GenericParameter, Parameter, Statement, TypeReference,
            VariableReference,
        },
        syntax::StringWithOffset,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn function_declaration() {
        let func = "fn distance from <a: Point> to <b: Point> -> Distance"
            .parse::<FunctionDeclaration>()
            .unwrap();
        assert_eq!(
            func,
            FunctionDeclaration {
                generic_parameters: vec![],
                name_parts: vec![
                    StringWithOffset::from("distance").at(3).into(),
                    StringWithOffset::from("from").at(12).into(),
                    Parameter {
                        less: 17,
                        name: StringWithOffset::from("a").at(18).into(),
                        ty: TypeReference {
                            name: StringWithOffset::from("Point").at(21).into(),
                            generic_parameters: Vec::new(),
                        },
                        greater: 26,
                    }
                    .into(),
                    StringWithOffset::from("to").at(28).into(),
                    Parameter {
                        less: 31,
                        name: StringWithOffset::from("b").at(32).into(),
                        ty: TypeReference {
                            name: StringWithOffset::from("Point").at(35).into(),
                            generic_parameters: Vec::new(),
                        },
                        greater: 40,
                    }
                    .into(),
                ],
                return_type: Some(TypeReference {
                    name: StringWithOffset::from("Distance").at(45).into(),
                    generic_parameters: Vec::new(),
                }),
                annotations: vec![],
                body: vec![],
                implicit_return: false,
            }
        );
    }

    #[test]
    fn function_with_single_line_body() {
        use crate::ast::Literal;

        let func = "fn test => 1".parse::<FunctionDeclaration>().unwrap();
        assert_eq!(
            func,
            FunctionDeclaration {
                generic_parameters: vec![],
                name_parts: vec![StringWithOffset::from("test").at(3).into(),],
                return_type: None,
                annotations: vec![],
                body: vec![Statement::Expression(
                    Literal::Integer {
                        value: "1".into(),
                        offset: 11,
                    }
                    .into()
                ),],
                implicit_return: true
            }
        );
    }

    #[test]
    fn generic_parameters() {
        let func = "fn<T> <x: T> -> T => x"
            .parse::<FunctionDeclaration>()
            .unwrap();
        assert_eq!(
            func,
            FunctionDeclaration {
                generic_parameters: vec![GenericParameter {
                    name: StringWithOffset::from("T").at(3).into(),
                    constraint: None
                }],
                name_parts: vec![Parameter {
                    less: 6,
                    name: StringWithOffset::from("x").at(7).into(),
                    ty: TypeReference {
                        name: StringWithOffset::from("T").at(10).into(),
                        generic_parameters: Vec::new(),
                    },
                    greater: 11,
                }
                .into()],
                return_type: Some(TypeReference {
                    name: StringWithOffset::from("T").at(16).into(),
                    generic_parameters: Vec::new(),
                }),
                annotations: vec![],
                body: vec![Statement::Expression(
                    VariableReference {
                        name: StringWithOffset::from("x").at(21).into(),
                    }
                    .into()
                ),],
                implicit_return: true
            }
        );

        let func = "fn<T: A> <x: T> -> T => x"
            .parse::<FunctionDeclaration>()
            .unwrap();
        assert_eq!(
            func,
            FunctionDeclaration {
                generic_parameters: vec![GenericParameter {
                    name: StringWithOffset::from("T").at(3).into(),
                    constraint: Some(TypeReference {
                        name: StringWithOffset::from("A").at(6).into(),
                        generic_parameters: vec![],
                    })
                }],
                name_parts: vec![Parameter {
                    less: 9,
                    name: StringWithOffset::from("x").at(10).into(),
                    ty: TypeReference {
                        name: StringWithOffset::from("T").at(13).into(),
                        generic_parameters: Vec::new(),
                    },
                    greater: 14,
                }
                .into(),],
                return_type: Some(TypeReference {
                    name: StringWithOffset::from("T").at(19).into(),
                    generic_parameters: Vec::new(),
                }),
                annotations: vec![],
                body: vec![Statement::Expression(
                    VariableReference {
                        name: StringWithOffset::from("x").at(24).into(),
                    }
                    .into()
                ),],
                implicit_return: true
            }
        );
    }
}
