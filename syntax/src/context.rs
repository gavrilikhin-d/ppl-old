use std::sync::Arc;

use serde_json::json;

use crate::{
    errors::{ExpectedRuleName, RuleNameNotCapitalized},
    parsers::ParseResult,
    patterns::Repeat,
    Pattern, Rule,
};

/// Action to be executed after parsing
pub type OnParsedAction =
    for<'s, 'c> fn(at: usize, res: ParseResult<'s>, context: &'c mut Context) -> ParseResult<'s>;

/// Helper function to make a rule transparent
fn transparent_ast() -> OnParsedAction {
    |_, mut res, _| {
        res.ast = res
            .ast
            .as_object()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .clone();
        res
    }
}

/// Rule with action to be executed after parsing
pub struct RuleWithAction {
    pub rule: Arc<Rule>,
    pub on_parsed: Option<OnParsedAction>,
}

impl From<Rule> for RuleWithAction {
    fn from(rule: Rule) -> Self {
        RuleWithAction {
            rule: Arc::new(rule),
            on_parsed: None,
        }
    }
}

/// Parsing context
pub struct Context {
    /// Parsing rules
    pub rules: Vec<RuleWithAction>,
}

impl Context {
    /// Create a new context without any rules
    pub fn new() -> Context {
        Context { rules: vec![] }
    }

    // Add a rule to the context
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule.into())
    }

    /// Find rule by name in the parsing context
    pub fn find_rule(&self, name: &str) -> Option<Arc<Rule>> {
        self.rules
            .iter()
            .map(|r| &r.rule)
            .find(|r| r.name == name)
            .cloned()
    }

    /// Get the callback to be called after parsing a rule
    pub fn on_parsed(&self, name: &str) -> Option<OnParsedAction> {
        self.rules
            .iter()
            .find(|r| r.rule.name == name)
            .and_then(|r| r.on_parsed)
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            rules: vec![
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Text".to_string(),
                        pattern: r"/[^\s*+?()|]+/".into(),
                    }),
                    on_parsed: transparent_ast().into(),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Regex".to_string(),
                        pattern: r"//[^/]+//".into(),
                    }),
                    on_parsed: transparent_ast().into(),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "RuleName".to_string(),
                        pattern: r"/[a-zA-Z0-9_]+/".into(),
                    }),
                    on_parsed: Some(|at, mut res, _| {
                        if res.has_errors() {
                            res.tree.children = vec![ExpectedRuleName { at: at.into() }.into()];
                            return res;
                        }

                        let rule_name = res.tree.tokens().next().unwrap();
                        let first_char = rule_name.chars().next().unwrap();
                        if !first_char.is_ascii_uppercase() {
                            res.tree.children =
                                vec![RuleNameNotCapitalized { at: at.into() }.into()]
                        }

                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "RuleReference".to_string(),
                        pattern: Pattern::RuleReference("RuleName".to_string()),
                    }),
                    on_parsed: Some(|_at, mut res, _| {
                        res.ast = json!({
                            "RuleReference": res.ast.get("RuleReference").unwrap().get("RuleName").unwrap()
                        });
                        if res.has_errors() {
                            res.delta = 0;
                        }
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Pattern".to_string(),
                        pattern: Pattern::RuleReference("Alternatives".to_string()),
                    }),
                    on_parsed: transparent_ast().into(),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Alternatives".to_string(),
                        pattern: vec![
                            Pattern::RuleReference("Sequence".to_string()),
                            Repeat::zero_or_more(
                                vec!["|".into(), Pattern::RuleReference("Sequence".to_string())]
                                    .into(),
                            )
                            .into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast()(at, res, context);
                        if res.has_errors() {
                            return res;
                        }

                        if !res.ast.is_array() {
                            return res;
                        }

                        let mut alts = Vec::new();
                        alts.push(res.ast.get(0).unwrap());

                        let arr = res.ast.get(1).unwrap().as_array().unwrap();
                        if arr.len() == 0 {
                            res.ast = alts[0].clone();
                            return res;
                        }

                        if arr.get(0).unwrap() == "|" {
                            alts.push(arr.get(1).unwrap());
                        } else {
                            for x in arr {
                                alts.push(x.get(1).unwrap());
                            }
                        }

                        res.ast = json!({ "Alternatives": alts });
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Sequence".to_string(),
                        pattern: Repeat::once_or_more(Pattern::RuleReference("Repeat".to_string()))
                            .into(),
                    }),
                    on_parsed: transparent_ast().into(),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Repeat".to_string(),
                        pattern: vec![
                            Pattern::RuleReference("AtomicPattern".to_string()),
                            Repeat::at_most_once(Pattern::Alternatives(
                                vec!["?".into(), "+".into(), "*".into()].into(),
                            ))
                            .into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast()(at, res, context);
                        if !res.ast.is_array() {
                            return res;
                        }

                        let pattern = res.ast.get(0).unwrap().clone();
                        let op = res.ast.get(1).unwrap();
                        if op.is_null() {
                            res.ast = pattern;
                            return res;
                        }
                        res.ast = match op.as_str().unwrap() {
                            "?" => json!({
                                "Repeat": {
                                    "pattern": pattern,
                                    "at_most": 1
                                }
                            }),
                            "+" => json!({
                                "Repeat": {
                                    "pattern": pattern,
                                    "at_least": 1,
                                }
                            }),
                            "*" => json!({
                                "Repeat": {
                                    "pattern": pattern,
                                }
                            }),
                            _ => unreachable!(),
                        };
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "AtomicPattern".to_string(),
                        pattern: Pattern::Alternatives(vec![
                            Pattern::RuleReference("PatternInParentheses".to_string()),
                            Pattern::RuleReference("RuleReference".to_string()),
                            Pattern::RuleReference("Regex".to_string()),
                            Pattern::RuleReference("Text".to_string()),
                        ]),
                    }),
                    on_parsed: transparent_ast().into(),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "PatternInParentheses".to_string(),
                        pattern: vec![
                            "(".into(),
                            Pattern::RuleReference("Pattern".to_string()),
                            ")".into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|_at, mut res, _| {
                        if res.has_errors() {
                            return res;
                        }

                        res.ast = res
                            .ast
                            .get("PatternInParentheses")
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .clone();
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Rule".to_string(),
                        pattern: vec![
                            Pattern::RuleReference("RuleName".to_string()),
                            ":".into(),
                            Pattern::RuleReference("Pattern".to_string()),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|_at, mut res, context| {
                        if res.has_errors() {
                            return res;
                        }
                        res.ast = res.ast.get("Rule").unwrap().clone();
                        res.ast = json!({
                            "name": res.ast.get(0).unwrap().get("RuleName").unwrap(),
                            "pattern": res.ast.get(2).unwrap()
                        });
                        let rule: Rule = serde_json::from_value(res.ast.clone()).unwrap();
                        context.add_rule(rule);
                        res
                    }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;

    use serde_json::json;

    use crate::{
        errors::{ExpectedRuleName, RuleNameNotCapitalized},
        parsers::{ParseResult, Parser},
        Context, ParseTree, Rule,
    };

    #[test]
    fn rule_name() {
        let mut ctx = Context::default();
        let rule_name = ctx.find_rule("RuleName").unwrap();
        assert_eq!(rule_name.name, "RuleName");
        assert_eq!(
            rule_name.parse("Foo", &mut ctx),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("RuleName").with("Foo"),
                ast: json!({"RuleName": "Foo"})
            }
        );
        assert_eq!(
            rule_name.parse("foo", &mut ctx),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("RuleName").with(RuleNameNotCapitalized { at: 0 }),
                ast: json!({"RuleName": "foo"})
            }
        );
        assert_eq!(
            rule_name.parse("", &mut ctx),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("RuleName").with(ExpectedRuleName { at: 0 }),
                ast: json!({ "RuleName": null })
            }
        );
    }

    #[test]
    fn rule_reference() {
        let mut context = Context::default();
        let r = context.find_rule("RuleReference").unwrap();
        assert_eq!(r.name, "RuleReference");
        assert_eq!(
            r.parse("Foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("RuleReference")
                    .with(ParseTree::named("RuleName").with("Foo")),
                ast: json!({"RuleReference": "Foo"})
            }
        );
        assert_eq!(
            r.parse("foo", &mut context),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("RuleReference")
                    .with(ParseTree::named("RuleName").with(RuleNameNotCapitalized { at: 0 })),
                ast: json!({"RuleReference": "foo"})
            }
        );
    }
    #[test]
    fn atomic_pattern() {
        let mut context = Context::default();
        let r = context.find_rule("AtomicPattern").unwrap();
        assert_eq!(r.name, "AtomicPattern");

        let tree_text = json!({
            "AtomicPattern": {
                "RuleReference": {
                    "RuleName": "Foo"
                }
            }
        })
        .to_string();
        assert_eq!(
            r.parse("Foo", &mut context),
            ParseResult {
                delta: 3,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "RuleReference": "Foo"
                })
            }
        );

        let tree_text = json!({
            "AtomicPattern": {
                "Text": "foo"
            }
        })
        .to_string();
        assert_eq!(
            r.parse("foo", &mut context),
            ParseResult {
                delta: 3,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!("foo")
            }
        );

        let tree_text = json!({
            "AtomicPattern": {
                "Regex": "/(xyz?)/"
            }
        })
        .to_string();
        assert_eq!(
            r.parse("/(xyz?)/", &mut context),
            ParseResult {
                delta: 8,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!("/(xyz?)/")
            }
        );

        let tree_text = json!({
            "AtomicPattern": {
                "PatternInParentheses": [
                    "(",
                    {
                        "Pattern": {
                            "Alternatives": {
                                "Sequence": {
                                    "Repeat": {
                                        "AtomicPattern": {
                                            "Text": {
                                                "value": "bar",
                                                "trivia": " "
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    {
                        "value": ")",
                        "trivia": " "
                    }
                ]
            }
        })
        .to_string();
        assert_eq!(
            r.parse("( bar )", &mut context),
            ParseResult {
                delta: 7,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!("bar")
            }
        );
    }

    #[test]
    fn sequence() {
        let mut context = Context::default();
        let r = context.find_rule("Sequence").unwrap();
        assert_eq!(r.name, "Sequence");

        let tree_text = json!({
            "Sequence": [
                {
                    "Repeat": {
                        "AtomicPattern": {
                            "RuleReference": {
                                "RuleName": "Foo"
                            }
                        }
                    }
                },
                {
                    "Repeat": [
                        {
                            "AtomicPattern": {
                                "Text": {
                                    "value": "bar",
                                    "trivia": " "
                                }
                            }
                        },
                        "?"
                    ]
                }
            ]
        })
        .to_string();
        assert_eq!(
            r.parse("Foo bar?", &mut context),
            ParseResult {
                delta: 8,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!([
                {
                    "RuleReference": "Foo"
                },
                {
                    "Repeat": {
                        "pattern": "bar",
                        "at_most": 1
                    }
                }])
            }
        )
    }

    #[test]
    fn alternatives() {
        let mut context = Context::default();
        let r = context.find_rule("Alternatives").unwrap();
        assert_eq!(r.name, "Alternatives");

        assert_eq!(
            r.parse("a | b", &mut context).ast,
            json!({
                "Alternatives": [
                    "a",
                    "b"
                ]
            })
        );

        assert_eq!(r.parse("a", &mut context).ast, json!("a"));

        assert_eq!(
            r.parse("a b | c d | e", &mut context).ast,
            json!({"Alternatives": [
                ["a", "b"],
                ["c", "d"],
                "e"
            ]})
        )
    }

    #[test]
    fn pattern() {
        let mut context = Context::default();
        let r = context.find_rule("Pattern").unwrap();
        assert_eq!(r.name, "Pattern");

        let tree_text = json!({
            "Pattern": {
                "Alternatives": {
                    "Sequence": {
                        "Repeat": [
                            {
                                "AtomicPattern": {
                                    "RuleReference": {
                                        "RuleName": "Foo"
                                    }
                                }
                            },
                            "?"
                        ]
                    }
                }
            },
        })
        .to_string();
        assert_eq!(
            r.parse("Foo?", &mut context),
            ParseResult {
                delta: 4,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "Repeat": {
                        "pattern": {
                            "RuleReference": "Foo"
                        },
                        "at_most": 1
                    }
                })
            }
        );

        let tree_text = json!({
            "Pattern": {
                "Alternatives": {
                    "Sequence": {
                        "Repeat": [
                            {
                                "AtomicPattern": {
                                    "Text": "foo"
                                }
                            },
                            "*"
                        ]
                    }
                }
            }
        })
        .to_string();
        assert_eq!(
            r.parse("foo*", &mut context),
            ParseResult {
                delta: 4,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "Repeat": {
                        "pattern": "foo"
                    }
                })
            }
        );

        let tree_text = json!({
            "Pattern": {
                "Alternatives": {
                    "Sequence": {
                        "Repeat": [
                            {
                                "AtomicPattern": {
                                    "PatternInParentheses": [
                                        "(",
                                        {
                                            "Pattern": {
                                                "Alternatives": {
                                                    "Sequence": {
                                                        "Repeat": {
                                                            "AtomicPattern": {
                                                                "Text": "bar"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        ")"
                                    ]
                                }
                            },
                            "+"
                        ]
                    }
                }
            }
        })
        .to_string();
        assert_eq!(
            r.parse("(bar)+", &mut context),
            ParseResult {
                delta: 6,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "Repeat": {
                        "pattern": "bar",
                        "at_least": 1
                    }
                })
            }
        );
    }

    #[test]
    fn rule() {
        let mut context = Context::default();
        let r = context.find_rule("Rule").unwrap();
        assert_eq!(r.name, "Rule");

        let tree_text = json!({
            "Rule": [
                { "RuleName": "Lol" },
                ":",
                {
                    "Pattern": {
                        "Alternatives": {
                            "Sequence": {
                                "Repeat": {
                                    "AtomicPattern": {
                                        "Text": {
                                            "value": "kek",
                                            "trivia": " "
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            ]
        })
        .to_string();
        assert_eq!(
            r.parse("Lol: kek", &mut context),
            ParseResult {
                delta: 8,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "name": "Lol",
                    "pattern": "kek"
                })
            }
        );
        assert_eq!(
            context.find_rule("Lol"),
            Some(Arc::new(Rule {
                name: "Lol".to_string(),
                pattern: "kek".into()
            }))
        )
    }
}
