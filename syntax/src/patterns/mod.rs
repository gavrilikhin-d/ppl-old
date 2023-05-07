mod named;
mod repeat;

use derive_more::From;
pub use named::*;
use regex::Regex;
pub use repeat::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    errors::Expected,
    parsers::{ParseResult, Parser},
    Context, ParseTree, ParseTreeNode, Token,
};

/// Possible patterns
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, From)]
pub enum Pattern {
    /// Reference to another rule
    #[from(ignore)]
    RuleReference(String),
    /// Group of patterns
    Group(Vec<Pattern>),
    /// Regex
    Regex(String),
    /// Pattern alternatives
    #[from(ignore)]
    Alternatives(Vec<Pattern>),
    /// Repeat pattern
    Repeat(Repeat),
    /// Adds name to the ast of pattern
    Named(Named),
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        Pattern::Regex(s.to_string())
    }
}

impl Parser for Pattern {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        match self {
            Pattern::Regex(r) => {
                // Find first not whitespace character
                let trivia_size = source[at..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .unwrap_or(source.len() - at);

                let re = Regex::new(format!("^{r}").as_str()).expect("Invalid regex");
                let m = re.find(&source[at + trivia_size..]).map(|m| m.as_str());
                ParseResult {
                    delta: m.map(|m| trivia_size + m.len()).unwrap_or(0),
                    tree: m
                        .map(|m| {
                            ParseTreeNode::from(Token {
                                value: m,
                                trivia: &source[at..at + trivia_size],
                            })
                            .into()
                        })
                        .unwrap_or_else(|| {
                            Expected {
                                expected: r.clone(),
                                at: at.into(),
                            }
                            .into()
                        }),
                    ast: m.into(),
                }
            }
            Pattern::RuleReference(name) => {
                let rule = context.find_rule(name).expect("Rule not found");
                let mut res = rule.parse_at(source, at, context);
                res.ast = json!({ name: res.ast });
                res
            }
            Pattern::Group(patterns) => {
                let mut delta = 0;
                let mut tree = ParseTree::empty();
                let mut asts = Vec::new();
                for pattern in patterns {
                    let result = pattern.parse_at(source, at + delta, context);
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
            Pattern::Alternatives(alts) => {
                let mut res = ParseResult::empty();
                for alt in alts {
                    res = alt.parse_at(source, at, context);
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            Pattern::Repeat(r) => r.parse_at(source, at, context),
            Pattern::Named(n) => n.parse_at(source, at, context),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use crate::{
        errors::Expected,
        parsers::{ParseResult, Parser},
        patterns::Named,
        Context, ParseTree, ParseTreeNode, Pattern,
    };

    #[test]
    fn regex() {
        let mut context = Context::default();
        let pattern: Pattern = r"[^\s]+".into();
        assert_eq!(
            pattern.parse("hello world", &mut context),
            ParseResult {
                delta: 5,
                tree: "hello".into(),
                ast: json!("hello")
            }
        );
    }

    #[test]
    fn alt() {
        let mut context = Context::default();
        let pattern = Pattern::Alternatives(vec!["a".into(), "b".into()]);
        assert_eq!(
            pattern.parse("a", &mut context),
            ParseResult {
                delta: 1,
                tree: "a".into(),
                ast: json!("a")
            }
        );
        assert_eq!(
            pattern.parse("b", &mut context),
            ParseResult {
                delta: 1,
                tree: "b".into(),
                ast: json!("b")
            }
        );
        assert_eq!(
            pattern.parse("c", &mut context),
            ParseResult {
                delta: 0,
                tree: Expected {
                    expected: "b".to_string(),
                    at: 0
                }
                .into(),
                ast: Value::Null
            }
        );
    }

    #[test]
    fn group() {
        let mut context = Context::default();
        let pattern = Pattern::Group(vec!["a".into(), "b".into()]);
        assert_eq!(
            pattern.parse("ab", &mut context),
            ParseResult {
                delta: 2,
                tree: vec!["a", "b"].into(),
                ast: json!(["a", "b"])
            }
        );
        assert_eq!(
            pattern.parse("b", &mut context),
            ParseResult {
                delta: 1,
                tree: vec![
                    ParseTreeNode::from(Expected {
                        expected: "a".to_string(),
                        at: 0
                    }),
                    "b".into()
                ]
                .into(),
                ast: json!([null, "b"])
            }
        );
        assert_eq!(
            pattern.parse("a", &mut context),
            ParseResult {
                delta: 1,
                tree: vec![
                    "a".into(),
                    ParseTreeNode::from(Expected {
                        expected: "b".to_string(),
                        at: 1
                    })
                ]
                .into(),
                ast: json!(["a", null])
            }
        );
        assert_eq!(
            pattern.parse("", &mut context),
            ParseResult {
                delta: 0,
                tree: vec![
                    ParseTreeNode::from(Expected {
                        expected: "a".to_string(),
                        at: 0
                    }),
                    Expected {
                        expected: "b".to_string(),
                        at: 0
                    }
                    .into()
                ]
                .into(),
                ast: json!([null, null])
            }
        )
    }

    #[test]
    fn rule_ref() {
        let mut context = Context::default();
        let pattern = Pattern::RuleReference("Regex".into());
        assert_eq!(
            pattern.parse("abc", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Regex").with("abc"),
                ast: json!({"Regex": "abc"})
            }
        )
    }

    #[test]
    fn named() {
        use crate::parsers::ParseResult;
        use crate::Context;

        let mut context = Context::default();
        let pattern: Pattern = Named {
            name: "name".to_string(),
            pattern: Box::new("[A-z][a-z]*".into()),
        }
        .into();
        assert_eq!(
            pattern.parse("John", &mut context),
            ParseResult {
                delta: 4,
                tree: "John".into(),
                ast: json!({"name": "John"}),
            }
        );
    }
}
