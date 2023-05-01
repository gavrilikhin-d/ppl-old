use serde_json::Value;

use crate::{errors::Error, ParseTree, Pattern};

/// Result of parsing
#[derive(Debug, PartialEq)]
pub struct ParseResult<'s> {
    /// Number of parsed characters
    pub delta: usize,
    /// Parse tree. May contain errors
    pub tree: ParseTree<'s>,
    /// AST
    pub ast: Value,
}

impl ParseResult<'_> {
    /// Create empty parse result
    pub fn empty() -> Self {
        Self {
            delta: 0,
            tree: ParseTree::empty(),
            ast: Value::Null,
        }
    }

    /// Does this result contain errors?
    pub fn has_errors(&self) -> bool {
        self.tree.has_errors()
    }

    /// Does this result contain no errors?
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Iterate over errors
    pub fn errors(&self) -> Box<dyn Iterator<Item = &dyn Error> + '_> {
        self.tree.errors()
    }
}

/// Parse source code starting at given position
pub trait Parser {
    /// Parse source code starting at given position
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s>;

    /// Parse source code from the beginning
    fn parse<'s>(&self, source: &'s str) -> ParseResult<'s> {
        self.parse_at(source, 0)
    }
}

/// Parse a list of patterns
pub fn parse_patterns_at<'s>(patterns: &[Pattern], source: &'s str, at: usize) -> ParseResult<'s> {
    let mut delta = 0;
    let mut tree = ParseTree::empty();
    let mut asts = Vec::new();
    for pattern in patterns {
        let result = pattern.parse_at(source, at + delta);
        delta += result.delta;
        tree.push(result.tree);
        asts.push(result.ast);
    }

    ParseResult {
        delta,
        tree: tree.flatten(),
        ast: if asts.len() != 1 {
            asts.into()
        } else {
            asts.pop().unwrap().into()
        },
    }
}
