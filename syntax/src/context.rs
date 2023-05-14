use std::sync::Arc;

use serde_json::json;

use crate::{
    errors::{ExpectedRuleName, RuleNameNotCapitalized},
    parsers::ParseResult,
    Pattern, Rule,
};

/// Action to be executed after parsing
pub type OnParsedAction =
    for<'s, 'c> fn(at: usize, res: ParseResult<'s>, context: &'c mut Context) -> ParseResult<'s>;

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
                Rule {
                    name: "Regex".to_string(),
                    pattern: r"[^\s]+".into(),
                }
                .into(),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "RuleName".to_string(),
                        pattern: r"[a-zA-Z0-9_]+".into(),
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
                        pattern: Pattern::Alternatives(vec![
                            Pattern::RuleReference("PatternInParentheses".to_string()),
                            Pattern::RuleReference("RuleReference".to_string()),
                            Pattern::RuleReference("Regex".to_string()),
                        ]),
                    }),
                    on_parsed: Some(|_, mut res, _| {
                        res.ast = res.ast.get("Pattern").unwrap().clone();
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "PatternInParentheses".to_string(),
                        pattern: vec![
                            regex::escape("(").into(),
                            Pattern::RuleReference("Pattern".to_string()),
                            regex::escape(")").into(),
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
    fn pattern() {
        let mut context = Context::default();
        let r = context.find_rule("Pattern").unwrap();
        assert_eq!(r.name, "Pattern");
        assert_eq!(
            r.parse("Foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Pattern").with(
                    ParseTree::named("RuleReference")
                        .with(ParseTree::named("RuleName").with("Foo"))
                ),
                ast: json!({
                    "RuleReference": "Foo"
                })
            }
        );
        assert_eq!(
            r.parse("foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Pattern").with(ParseTree::named("Regex").with("foo")),
                ast: json!({"Regex": "foo"})
            }
        );

        let tree_text = json!({
            "Pattern": {
                "PatternInParentheses": [
                    "(",
                    {
                        "Pattern": {
                            "Regex": {
                                "value": "bar",
                                "trivia": " "
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
                ast: json!({"Regex": "bar"})
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
                        "Regex": {
                            "value": "kek",
                            "trivia": " "
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
                    "pattern": {
                        "Regex": "kek"
                    }
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
