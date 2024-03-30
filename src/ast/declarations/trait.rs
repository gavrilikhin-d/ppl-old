extern crate ast_derive;
use ast_derive::AST;

use crate::{
    ast::TypeReference,
    syntax::{
        error::ParseError, Context, Identifier, Keyword, Lexer, Parse, Ranged, StartsHere, Token,
    },
};

use super::FunctionDeclaration;

/// Declaration of trait
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TraitDeclaration {
    /// Keyword `trait`
    pub keyword: Keyword<"trait">,
    /// Name of trait
    pub name: Identifier,
    /// Supertraits for this trait
    pub supertraits: Vec<TypeReference>,
    /// Associated functions
    pub functions: Vec<FunctionDeclaration>,
}

impl Ranged for TraitDeclaration {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.functions
            .last()
            .map_or_else(|| self.name.end(), |s| s.end())
    }
}

impl StartsHere for TraitDeclaration {
    /// Check that type declaration may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::Trait)
    }
}

impl Parse for TraitDeclaration {
    type Err = ParseError;

    /// Parse trait declaration
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"trait">()?;

        let name = context.consume_id()?;

        let colon = context.lexer.consume(Token::Colon)?;

        let supertraits = context.parse_comma_separated(TypeReference::parse);

        let error_range = keyword.start()..colon.start();
        let functions = context.parse_block(FunctionDeclaration::parse, error_range)?;

        Ok(TraitDeclaration {
            keyword,
            name,
            supertraits,
            functions,
        })
    }
}
