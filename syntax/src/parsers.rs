use crate::{ParseTree, Pattern, Rule};

/// Result of parsing
#[derive(Debug, PartialEq)]
pub struct ParseResult<'s> {
    /// Number of parsed characters
    pub delta: usize,
    /// Parse tree. May contain errors
    pub tree: ParseTree<'s>,
}

impl ParseResult<'_> {
    /// Create empty parse result
    pub fn empty() -> Self {
        Self {
            delta: 0,
            tree: ParseTree::Tree(vec![]),
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
}

/// Parse source code starting at given position
pub trait Parser {
    /// Parse source code starting at given position
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s>;
}

/// Create default parsing rules
pub fn create_default_rules() -> Vec<Rule> {
    vec![Rule {
        name: "Regex".to_string(),
        patterns: vec![Pattern::Regex(r"[^\s]+".to_string())],
    }]
}

/// Parse a list of patterns
pub fn parse_patterns_at<'s>(patterns: &[Pattern], source: &'s str, at: usize) -> ParseResult<'s> {
    let mut delta = 0;
    let mut trees = Vec::new();
    for pattern in patterns {
        let result = pattern.parse_at(source, at + delta);
        delta += result.delta;
        trees.push(result.tree);
    }
    ParseResult {
        delta,
        tree: trees.into(),
    }
}
