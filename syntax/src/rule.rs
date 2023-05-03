use serde_json::json;

use crate::{
    context,
    parsers::{ParseResult, Parser},
    ParseTree, Pattern,
};

/// Syntax rule
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Pattern to parse
    pub pattern: Pattern,
}

impl Parser for Rule {
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s> {
        let res = self.pattern.parse_at(source, at);
        let res = ParseResult {
            delta: res.delta,
            tree: ParseTree::named(self.name.clone()).with(res.tree).flatten(),
            ast: json!({ &self.name: res.ast }),
        };
        if let Some(on_parsed) = context::on_parsed(&self.name) {
            on_parsed(at, res)
        } else {
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ParseTree;

    use super::*;

    #[test]
    fn test_parse_rule() {
        let rule = Rule {
            name: "Test".to_string(),
            pattern: r"[^\s]+".into(),
        };
        assert_eq!(
            rule.parse_at("Hello World", 0),
            ParseResult {
                delta: 5,
                tree: ParseTree::named("Test").with("Hello"),
                ast: json!({"Test": "Hello"})
            }
        );
    }
}
