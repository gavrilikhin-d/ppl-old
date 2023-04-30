use crate::{
    parsers::{self, ParseResult, Parser},
    Pattern,
};

/// Type of rule name
pub type RuleName = String;

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    /// Rule name
    pub name: RuleName,
    /// Rule patterns
    pub patterns: Vec<Pattern>,
}

impl Parser for Rule {
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s> {
        let res = parsers::parse_patterns_at(&self.patterns, source, at);
        ParseResult {
            delta: res.delta,
            tree: res.tree.with_name(self.name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseTree;

    use super::*;

    #[test]
    fn test_parse_rule() {
        let rule = Rule {
            name: "Test".to_string(),
            patterns: vec![r"[^\s]+".into()],
        };
        assert_eq!(
            rule.parse_at("Hello World", 0),
            ParseResult {
                delta: 5,
                tree: ParseTree::named("Test").with("Hello")
            }
        );
    }
}
