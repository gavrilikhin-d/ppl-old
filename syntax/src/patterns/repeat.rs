use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    parsers::{ParseResult, Parser},
    Context, ParseTree,
};

use super::Pattern;

/// Repeat pattern
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Repeat {
    /// Pattern to repeat
    pub pattern: Box<Pattern>,
    /// Minimum number of repetitions
    #[serde(default)]
    pub at_least: usize,
    /// Maximum number of repetitions
    pub at_most: Option<usize>,
}

impl Repeat {
    /// Repeat pattern zero or more times (`*`)
    pub fn zero_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: None,
        }
    }

    /// Repeat pattern once or more times (`+`)
    pub fn once_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 1,
            at_most: None,
        }
    }

    /// Repeat pattern at most once (`?`)
    pub fn at_most_once(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: Some(1),
        }
    }
}

impl Parser for Repeat {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        debug_assert!(self.at_least <= self.at_most.unwrap_or(usize::MAX));

        let mut delta = 0;
        let mut tree = ParseTree::empty();
        let mut asts = Vec::new();
        for _ in 0..self.at_least {
            let res = self.pattern.parse_at(source, at + delta, context);
            delta += res.delta;
            tree.push(res.tree);
            asts.push(res.ast);
        }

        if tree.is_ok() {
            for _ in self.at_least..self.at_most.unwrap_or(usize::MAX) {
                let res = self.pattern.parse_at(source, at + delta, context);
                if res.is_ok() {
                    delta += res.delta;
                    tree.push(res.tree);
                    asts.push(res.ast);
                } else {
                    break;
                }
            }
        }

        ParseResult {
            delta,
            tree: tree.flatten(),
            ast: if self.at_most == Some(1) {
                if asts.len() == 1 {
                    asts.into_iter().next().unwrap()
                } else {
                    json!(null)
                }
            } else {
                asts.into()
            },
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::{
        errors::Expected,
        parsers::{ParseResult, Parser},
        Context,
    };

    use super::Repeat;

    #[test]
    fn at_most_once() {
        let mut context = Context::default();
        let pattern = Repeat::at_most_once("a".into());
        assert_eq!(
            pattern.parse("", &mut context),
            ParseResult::empty().with_ast(json!(null))
        );
        assert_eq!(
            pattern.parse("a", &mut context),
            ParseResult {
                delta: 1,
                tree: "a".into(),
                ast: "a".into()
            }
        );
        assert_eq!(
            pattern.parse("aa", &mut context),
            ParseResult {
                delta: 1,
                tree: "a".into(),
                ast: "a".into()
            }
        )
    }

    #[test]
    fn zero_or_more() {
        let mut context = Context::default();
        let pattern = Repeat::zero_or_more("a".into());
        assert_eq!(
            pattern.parse("", &mut context),
            ParseResult::empty().with_ast(json!([]))
        );
        assert_eq!(
            pattern.parse("a", &mut context),
            ParseResult {
                delta: 1,
                tree: "a".into(),
                ast: vec!["a"].into()
            }
        );
        assert_eq!(
            pattern.parse("aa", &mut context),
            ParseResult {
                delta: 2,
                tree: vec!["a", "a"].into(),
                ast: json!(["a", "a"])
            }
        );
    }

    #[test]
    fn once_or_more() {
        let mut context = Context::default();
        let pattern = Repeat::once_or_more("a".into());
        assert_eq!(
            pattern.parse("", &mut context),
            ParseResult {
                delta: 0,
                tree: Expected {
                    expected: "a".to_string(),
                    at: 0
                }
                .into(),
                ast: json!([null])
            }
        );
        assert_eq!(
            pattern.parse("a", &mut context),
            ParseResult {
                delta: 1,
                tree: "a".into(),
                ast: vec!["a"].into()
            }
        );
        assert_eq!(
            pattern.parse("aa", &mut context),
            ParseResult {
                delta: 2,
                tree: vec!["a", "a"].into(),
                ast: json!(["a", "a"])
            }
        );
    }
}
