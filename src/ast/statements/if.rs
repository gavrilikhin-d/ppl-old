extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Keyword, Ranged, StartsHere};

use super::Statement;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElseIf {
    /// Keyword `else`
    pub else_keyword: Keyword<"else">,
    /// Keyword `if`
    pub if_keyword: Keyword<"if">,
    /// Condition of else-if statement
    pub condition: Expression,
    /// Body of else-if statement
    pub body: Vec<Statement>,
}

impl Ranged for ElseIf {
    fn start(&self) -> usize {
        self.else_keyword.start()
    }

    fn end(&self) -> usize {
        self.body
            .last()
            .map_or_else(|| self.condition.end(), |s| s.end())
    }
}

/// AST for else block
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Else {
    /// Keyword `else`
    pub keyword: Keyword<"else">,
    /// Body of else statement
    pub body: Vec<Statement>,
}

impl Ranged for Else {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.body
            .last()
            .map_or_else(|| self.keyword.end(), |s| s.end())
    }
}

/// AST for if-statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct If {
    /// Keyword `if`
    pub keyword: Keyword<"if">,
    /// Condition of if-statement
    pub condition: Expression,
    /// Body of if-statement
    pub body: Vec<Statement>,
    /// Else-if statements
    pub else_ifs: Vec<ElseIf>,
    /// Else block
    pub else_block: Option<Else>,
}

impl Ranged for If {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        if let Some(else_block) = &self.else_block {
            else_block.end()
        } else if let Some(else_if) = self.else_ifs.last() {
            else_if.end()
        } else {
            self.body
                .last()
                .map_or_else(|| self.condition.end(), |s| s.end())
        }
    }
}

impl StartsHere for If {
    /// Check that assignment may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::If)
    }
}

impl Parse for If {
    type Err = ParseError;

    /// Parse assignment using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"if">()?;

        let condition = Expression::parse(context)?;

        let colon = context.lexer.consume(Token::Colon)?;

        let error_range = keyword.start()..colon.start();
        let body = context.parse_block(Statement::parse, error_range)?;

        let mut else_ifs = Vec::new();
        let mut else_block = None;
        while let Ok(else_keyword) = context.consume_keyword::<"else">() {
            if let Ok(if_keyword) = context.consume_keyword::<"if">() {
                let condition = Expression::parse(context)?;

                context.lexer.consume(Token::Colon)?;
                let error_range = else_keyword.start()..if_keyword.end();
                let body = context.parse_block(Statement::parse, error_range)?;
                else_ifs.push(ElseIf {
                    else_keyword,
                    if_keyword,
                    condition,
                    body,
                });
            } else {
                context.lexer.consume(Token::Colon)?;
                else_block = Some(Else {
                    keyword: else_keyword,
                    body: context.parse_block(Statement::parse, else_keyword.range())?,
                });
                break;
            }
        }

        Ok(If {
            keyword,
            condition,
            body,
            else_ifs,
            else_block,
        })
    }
}
