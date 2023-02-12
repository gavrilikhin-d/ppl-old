use crate::ast::Statement;

use super::{
    error::{LexerError, ParseError},
    PrecedenceGroups, StringWithOffset, Token,
};

/// Context for parsing
pub struct Context<Lexer: super::Lexer> {
    /// Lexer to use for parsing
    pub lexer: Lexer,
    /// Currently active precedence groups for operators
    pub precedence_groups: PrecedenceGroups,
}

impl<Lexer: super::Lexer> Context<Lexer> {
    /// Consume end of line (newline or eof)
    pub fn consume_eol(&mut self) -> Result<Option<StringWithOffset>, LexerError> {
        if self.lexer.peek().is_some() {
            return Ok(Some(self.lexer.consume(Token::Newline)?));
        }
        Ok(None)
    }

    /// Consume id or escaped id
    pub fn consume_id(&mut self) -> Result<StringWithOffset, LexerError> {
        if let Some(Token::EscapedId(id)) = self.lexer.peek() {
            self.lexer.next();
            return Ok(StringWithOffset::from(id).at(self.lexer.span().start + 1));
        }
        self.lexer.consume(Token::Id)
    }

    /// Parse block of items
    pub fn parse_block<T>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let indentation = self.lexer.indentation() + 1;

        self.lexer.consume(Token::Newline)?;
        self.lexer.skip_indentation();

        let mut stmts = Vec::new();
        while self.lexer.indentation() == indentation && self.lexer.peek().is_some() {
            stmts.push(parse(self)?);
            self.lexer.skip_indentation();
        }
        Ok(stmts)
    }
}

impl<'l, Lexer: super::Lexer> Context<Lexer> {
    /// Create new context with default precedence groups
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            precedence_groups: PrecedenceGroups::default(),
        }
    }
}

/// Trait for parsing using context.lexer
pub trait Parse
where
    Self: Sized,
{
    type Err;

    /// Parse starting from current context.lexer state
    fn parse(context: &mut Context<impl super::Lexer>) -> Result<Self, Self::Err>;
}

/// Trait for checking that current context.lexer state is 100% start of a node
pub trait StartsHere {
    /// Check if current context.lexer state is 100% start of this node
    fn starts_here(context: &mut Context<impl super::Lexer>) -> bool;
}
