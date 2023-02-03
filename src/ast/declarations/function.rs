extern crate ast_derive;
use ast_derive::AST;

use derive_more::From;

use crate::{
    ast::{Annotation, Statement, Expression, TypeReference},
    syntax::{error::ParseError, Lexer, Parse, StartsHere, StringWithOffset, Token, Context, OperatorKind, Ranged},
};

/// Parameter of function
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Parameter {
    /// Parameter's name
    pub name: StringWithOffset,
    /// Parameter's type
    pub ty: TypeReference,
}

impl Ranged for Parameter {
	/// Get range of parameter
	fn range(&self) -> std::ops::Range<usize> {
		self.name.range().start..self.ty.range().end
	}
}

impl Parse for Parameter {
    type Err = ParseError;

    /// Parse parameter using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let name = context.lexer
            .consume(Token::Id)
            .ok()
            .unwrap_or_else(|| StringWithOffset::from("").at(context.lexer.span().end));

        context.lexer.consume(Token::Colon)?;

        let ty = TypeReference::parse(context)?;

        Ok(Parameter { name, ty })
    }
}

/// Cell of function
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum FunctionNamePart {
    Text(StringWithOffset),
    Parameter{
		less: usize,
		parameter: Parameter,
		greater: usize,
	},
}

impl Ranged for FunctionNamePart {
	/// Get range of function name part
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			FunctionNamePart::Text(s) => s.range(),
			FunctionNamePart::Parameter{less, greater, ..} => *less..*greater + 1,
		}
	}
}

impl Parse for FunctionNamePart {
    type Err = ParseError;

    /// Parse function name part using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let token = context.lexer.consume_one_of(&[
			Token::Id,
			Token::Less, Token::Greater,
			Token::Operator(OperatorKind::Prefix),
			Token::Operator(OperatorKind::Infix),
			Token::Operator(OperatorKind::Postfix)
		])?;
        match token {
            Token::Id | Token::Greater | Token::Operator(_)
				=> Ok(context.lexer.string_with_offset().into()),
            Token::Less => {
				// '<' here is an operator
				if context.lexer.peek() == Some(Token::Less) {
					return Ok(context.lexer.string_with_offset().into())
				}

				let less = context.lexer.span().start;

                let parameter = Parameter::parse(context)?;

                let greater = context.lexer.consume(Token::Greater)?.start();

                Ok(
					FunctionNamePart::Parameter{
						less,
						parameter,
						greater,
					}
				)
            }
            _ => unreachable!("consume_one_of returned unexpected token"),
        }
    }
}

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct FunctionDeclaration {
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
        context.lexer.consume(Token::Fn)?;

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
            body = context.parse_block()?;
        }
		else {
			context.consume_eol()?;
		}

        Ok(FunctionDeclaration {
            name_parts,
            return_type,
            body,
			implicit_return,
            annotations: vec![],
        })
    }
}

#[test]
fn test_function_declaration() {
    let func = "fn distance from <a: Point> to <b: Point> -> Distance"
        .parse::<FunctionDeclaration>()
        .unwrap();
    assert_eq!(
        func,
        FunctionDeclaration {
            name_parts: vec![
                StringWithOffset::from("distance").at(3).into(),
                StringWithOffset::from("from").at(12).into(),
                Parameter {
                    name: StringWithOffset::from("a").at(18).into(),
                    ty: TypeReference {
						name: StringWithOffset::from("Point").at(21).into()
					},
                }
                .into(),
                StringWithOffset::from("to").at(28).into(),
                Parameter {
                    name: StringWithOffset::from("b").at(32).into(),
                    ty: TypeReference {
						name: StringWithOffset::from("Point").at(35).into()
					},
                }
                .into(),
            ],
            return_type: Some(TypeReference {
				name: StringWithOffset::from("Distance").at(45).into()
			}),
            annotations: vec![],
            body: vec![],
			implicit_return: false,
        }
    );
}

#[test]
fn test_function_with_single_line_body() {
    use crate::ast::Literal;

    let func = "fn test => 1".parse::<FunctionDeclaration>().unwrap();
    assert_eq!(
        func,
        FunctionDeclaration {
            name_parts: vec![StringWithOffset::from("test").at(3).into(),],
            return_type: None,
            annotations: vec![],
            body: vec![
                Statement::Expression(
                    Literal::Integer {
                        value: "1".into(),
                        offset: 11,
                    }
                    .into()
                ),
            ],
			implicit_return: true
        }
    );
}
