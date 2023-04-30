mod repeat;

use derive_more::From;
use regex::Regex;
pub use repeat::*;

use crate::{
    context,
    errors::Expected,
    parsers::{parse_patterns_at, ParseResult, Parser},
};

/// Possible patterns
#[derive(Debug, PartialEq, Clone, From)]
pub enum Pattern {
    /// Reference to another rule
    RuleReference(String),
    /// Group of patterns
    Group(Vec<Pattern>),
    /// Regex
    Regex(String),
    /// Pattern alternatives
    Alternatives(Vec<Pattern>),
    /// Repeat pattern
    #[from]
    Repeat(Repeat),
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        Pattern::Regex(s.to_string())
    }
}

impl Parser for Pattern {
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s> {
        match self {
            Pattern::Regex(r) => {
                let re = Regex::new(format!("^{r}").as_str()).expect("Invalid regex");
                let m = re.find(&source[at..]).map(|m| m.as_str());
                ParseResult {
                    delta: m.map(|m| m.len()).unwrap_or(0),
                    tree: m.map(|m| m.into()).unwrap_or_else(|| {
                        Expected {
                            expected: r.clone(),
                            at: at.into(),
                        }
                        .into()
                    }),
                }
            }
            Pattern::RuleReference(name) => {
                let rule = context::find_rule(name).expect("Rule not found");
                rule.parse_at(source, at)
            }
            Pattern::Group(patterns) => parse_patterns_at(patterns, source, at),
            Pattern::Alternatives(alts) => {
                let mut res = ParseResult::empty();
                for alt in alts {
                    res = alt.parse_at(source, at);
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            Pattern::Repeat(r) => r.parse_at(source, at),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        errors::Expected,
        parsers::{ParseResult, Parser},
        IntoParseTreeNode, Pattern,
    };

    #[test]
    fn regex() {
        let pattern: Pattern = r"[^\s]+".into();
        assert_eq!(
            pattern.parse_at("hello world", 0),
            ParseResult {
                delta: 5,
                tree: "hello".into()
            }
        );
    }

    #[test]
    fn alt() {
        let pattern = Pattern::Alternatives(vec!["a".into(), "b".into()]);
        assert_eq!(
            pattern.parse_at("a", 0),
            ParseResult {
                delta: 1,
                tree: "a".into()
            }
        );
        assert_eq!(
            pattern.parse_at("b", 0),
            ParseResult {
                delta: 1,
                tree: "b".into()
            }
        );
        assert_eq!(
            pattern.parse_at("c", 0),
            ParseResult {
                delta: 0,
                tree: Expected {
                    expected: "b".to_string(),
                    at: 0.into()
                }
                .into()
            }
        );
    }

    #[test]
    fn group() {
        let pattern = Pattern::Group(vec!["a".into(), "b".into()]);
        assert_eq!(
            pattern.parse_at("ab", 0),
            ParseResult {
                delta: 2,
                tree: vec!["a", "b"].into()
            }
        );
        assert_eq!(
            pattern.parse_at("b", 0),
            ParseResult {
                delta: 1,
                tree: vec![
                    Expected {
                        expected: "a".to_string(),
                        at: 0.into()
                    }
                    .into_parse_tree_node(),
                    "b".into()
                ]
                .into()
            }
        );
        assert_eq!(
            pattern.parse_at("a", 0),
            ParseResult {
                delta: 1,
                tree: vec![
                    "a".into(),
                    Expected {
                        expected: "b".to_string(),
                        at: 1.into()
                    }
                    .into_parse_tree_node()
                ]
                .into()
            }
        );
        assert_eq!(
            pattern.parse_at("", 0),
            ParseResult {
                delta: 0,
                tree: vec![
                    Expected {
                        expected: "a".to_string(),
                        at: 0.into()
                    }
                    .into_parse_tree_node(),
                    Expected {
                        expected: "b".to_string(),
                        at: 0.into()
                    }
                    .into_parse_tree_node()
                ]
                .into()
            }
        )
    }
}
