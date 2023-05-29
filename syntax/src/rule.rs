use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    parsers::{ParseResult, Parser},
    patterns::Sequence,
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

impl Rule {
    /// Create a new rule with a name and a pattern
    pub fn new(name: impl Into<String>, pattern: impl Into<Pattern>) -> Self {
        Self {
            name: name.into(),
            pattern: pattern.into(),
        }
    }
}

impl Parser for Rule {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        let mut res = self.pattern.parse_at(source, at, context);
        res.tree = ParseTree::named(self.name.clone()).with(res.tree).flatten();

        if res.has_errors() {
            return res;
        }

        if !matches!(
            self.pattern,
            Pattern::Sequence(Sequence {
                action: Some(_),
                ..
            })
        ) {
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
    use crate::{action, patterns::Repeat};

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

    #[test]
    fn deserialize_and_parse_rule_with_action() {
        let mut context = Context::default();
        let rule = context.find_rule("Rule").unwrap();
        assert_eq!(rule.name, "Rule");

        assert_eq!(
            rule.parse("List: '(' <letters: x*> ')' => letters", &mut context)
                .ast,
            json!({
                "name": "List",
                "pattern": {
                    "Sequence": {
                        "patterns": [
                            '(',
                            {
                                "Named": {
                                    "name": "letters",
                                    "pattern": {
                                        "Repeat": {
                                            "pattern": "x",
                                        }
                                    }
                                }
                            },
                            ')'
                        ],
                        "action": {
                            "Variable": "letters"
                        }
                    }
                }
            })
        );

        let rule = context.find_rule("List").unwrap();
        assert_eq!(
            rule.as_ref(),
            &Rule::new(
                "List",
                Sequence::new(
                    vec![
                        '('.into(),
                        ("letters", Repeat::zero_or_more("x").into()).into(),
                        ')'.into()
                    ]
                    .into(),
                    action::reference("letters")
                )
            )
        );
        assert_eq!(rule.parse("(x x)", &mut context).ast, json!(["x", "x"]))
    }
}
