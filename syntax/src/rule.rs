use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    parsers::{ParseResult, Parser},
    Context, ParseTree, Pattern,
};

/// Syntax rule
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Pattern to parse
    pub pattern: Pattern,
}

impl Parser for Rule {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        let mut res = self.pattern.parse_at(source, at, context);
        res.tree = ParseTree::named(self.name.clone()).with(res.tree).flatten();
        res.ast = json!({ &self.name: res.ast });
        if let Some(on_parsed) = context.on_parsed(&self.name) {
            on_parsed(at, res, context)
        } else {
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule() {
        let mut context = Context::new();
        let rule = Rule {
            name: "Test".to_string(),
            pattern: r"[^\s]+".into(),
        };
        assert_eq!(
            rule.parse("Hello World", &mut context),
            ParseResult {
                delta: 5,
                tree: ParseTree::named("Test").with("Hello"),
                ast: json!({"Test": "Hello"})
            }
        );
    }
}
