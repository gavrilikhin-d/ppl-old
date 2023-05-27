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

        if res.has_errors() {
            return res;
        }

        if let Pattern::Sequence(seq) = &self.pattern {
            let mut ast = serde_json::Map::new();
            seq.iter()
                .enumerate()
                .filter_map(|(i, p)| match p {
                    Pattern::Named(_) => Some(i),
                    _ => None,
                })
                .for_each(|i| {
                    let mut named = res.ast.get_mut(i).unwrap().as_object_mut().unwrap();
                    ast.append(&mut named);
                });
            res.ast = json!({ &self.name: ast });
        } else {
            res.ast = json!({ &self.name: res.ast.take() });
        }

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

    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_rule() {
        let mut context = Context::new();
        let rule = Rule {
            name: "Test".to_string(),
            pattern: r"/[^\s]+/".into(),
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

    #[test]
    fn test_deserialize_rule_with_sequence() {
        let mut context = Context::default();
        let rule = context.find_rule("Rule").unwrap();
        assert_eq!(rule.name, "Rule");

        let tree_text = json!({
            "Rule": [
                { "RuleName": "X" },
                ":",
                {
                    "Pattern": {
                        "Alternatives": {
                            "Sequence": [
                                {
                                    "Repeat": {
                                        "AtomicPattern": {
                                            "Text": {
                                                "trivia": " ",
                                                "value": "a",
                                            }
                                        }
                                    }
                                },
                                {
                                    "Repeat": {
                                        "AtomicPattern": {
                                            "Text": {
                                                "trivia": " ",
                                                "value": "b",
                                            }
                                        }
                                    }
                                }
                            ]
                        }
                    }
                }
            ]
        })
        .to_string();
        assert_eq!(
            rule.parse("X: a b", &mut context),
            ParseResult {
                delta: 6,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!(
                    {
                        "name": "X",
                        "pattern": ["a", "b"]
                    }
                )
            }
        );

        let rule = context.find_rule("X").unwrap();
        assert_eq!(rule.name, "X");
        assert_eq!(rule.pattern, vec!["a".into(), "b".into()].into());
    }

    #[test]
    fn test_deserialize_rule_with_regex() {
        let mut context = Context::default();
        let rule = context.find_rule("Rule").unwrap();
        assert_eq!(rule.name, "Rule");

        let tree_text = json!({
            "Rule": [
                { "RuleName": "X" },
                ":",
                {
                    "Pattern": {
                        "Alternatives": {
                            "Sequence": [
                                {
                                    "Repeat": {
                                        "AtomicPattern": {
                                            "Regex": {
                                                "trivia": " ",
                                                "value": "/ab?c/",
                                            }
                                        }
                                    }
                                },
                            ]
                        }
                    }
                }
            ]
        })
        .to_string();
        assert_eq!(
            rule.parse("X: /ab?c/", &mut context),
            ParseResult {
                delta: 9,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!(
                    {
                        "name": "X",
                        "pattern": "/ab?c/"
                    }
                )
            }
        );

        let rule = context.find_rule("X").unwrap();
        assert_eq!(rule.name, "X");
        assert_eq!(rule.pattern, r"/ab?c/".into());
    }
}
