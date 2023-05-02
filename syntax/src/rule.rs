use crate::{
    parsers::{ParseResult, Parser},
    ParseTree, Pattern,
};

/// Syntax rule
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Pattern to parse
    pub pattern: Pattern,
    /// Callback to be called after parsing
    pub on_parsed: Option<Box<dyn Sync + Send + Fn(usize, ParseResult) -> ParseResult>>,
}

impl Parser for Rule {
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s> {
        let res = self.pattern.parse_at(source, at);
        let res = ParseResult {
            delta: res.delta,
            tree: ParseTree::named(self.name.clone()).with(res.tree).flatten(),
            ast: res.ast,
        };
        if let Some(on_parsed) = &self.on_parsed {
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
            on_parsed: None,
        };
        assert_eq!(
            rule.parse_at("Hello World", 0),
            ParseResult {
                delta: 5,
                tree: ParseTree::named("Test").with("Hello"),
                ast: json!("Hello")
            }
        );
    }
}
