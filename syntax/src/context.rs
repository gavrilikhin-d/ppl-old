use std::sync::Arc;

use serde_json::json;

use crate::{
    action::{reference, Action},
    parsers::ParseResult,
    patterns::{rule_ref, Repeat, Sequence},
    Pattern, Rule,
};

/// Action to be executed after parsing
pub type OnParsedAction =
    for<'s, 'c> fn(at: usize, res: ParseResult<'s>, context: &'c mut Context) -> ParseResult<'s>;

/// Helper function to make a rule transparent
fn transparent_ast<'s>(
    _at: usize,
    mut res: ParseResult<'s>,
    _context: &mut Context,
) -> ParseResult<'s> {
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

/// Helper function to make a rule transparent and remove quotes
fn without_quotes<'s>(
    at: usize,
    mut res: ParseResult<'s>,
    context: &mut Context,
) -> ParseResult<'s> {
    res = transparent_ast(at, res, context);
    let s = res.ast.as_str().unwrap();
    res.ast = json!(s[1..s.len() - 1]);
    res
}

/// Rule with action to be executed after parsing
pub struct RuleWithAction {
    pub rule: Arc<Rule>,
    pub on_parsed: Option<OnParsedAction>,
}

impl RuleWithAction {
    /// Create a new rule with an action
    pub fn new(rule: Rule, on_parsed: OnParsedAction) -> Self {
        RuleWithAction {
            rule: Arc::new(rule),
            on_parsed: Some(on_parsed),
        }
    }
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
    /// Root pattern
    pub root: Pattern,
}

impl Context {
    /// Create a new context without any rules
    pub fn new() -> Context {
        Context {
            rules: vec![],
            root: "".into(),
        }
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
            root: Pattern::RuleReference("Rule".to_string()),
            rules: vec![
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Char".to_string(),
                        pattern: r"/'.'/".into(),
                    }),
                    on_parsed: Some(without_quotes),
                },
                RuleWithAction::new(
                    Rule::new(
                        "Integer",
                        Sequence::new(
                            vec![("value", r"/[0-9]+/".into()).into()],
                            Action::Return(reference("value")),
                        ),
                    ),
                    |_, mut res, _| {
                        let str = res.ast.as_str().unwrap();
                        if let Ok(i) = str.parse::<i64>() {
                            res.ast = i.into();
                        } else {
                            res.ast = json!({ "Integer": str });
                        }
                        res
                    },
                ),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "String".to_string(),
                        pattern: "/\"([^\"\\\\]|\\.)*\"/".into(),
                    }),
                    on_parsed: Some(without_quotes),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Text".to_string(),
                        pattern: Pattern::Alternatives(vec![
                            Pattern::RuleReference("Char".to_string()),
                            Pattern::RuleReference("String".to_string()),
                            r"/[^\s*+?()|<:>{}=]+/".into(),
                        ]),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Regex".to_string(),
                        pattern: r"//[^/]+//".into(),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "RuleName".to_string(),
                        pattern: r"/[A-Z][a-zA-Z0-9]*/".into(),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                Rule {
                    name: "RuleReference".to_string(),
                    pattern: Pattern::RuleReference("RuleName".to_string()),
                }
                .into(),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Pattern".to_string(),
                        pattern: Pattern::RuleReference("Alternatives".to_string()),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Alternatives".to_string(),
                        pattern: vec![
                            ("head", Pattern::RuleReference("Sequence".to_string())).into(),
                            (
                                "tail",
                                Repeat::zero_or_more(vec![
                                    "|".into(),
                                    ("sequence", Pattern::RuleReference("Sequence".to_string()))
                                        .into(),
                                ])
                                .into(),
                            )
                                .into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast(at, res, context);

                        let mut alts = Vec::new();
                        alts.push(res.ast.get("head").unwrap());

                        let arr = res.ast.get("tail").unwrap().as_array().unwrap();
                        if arr.len() == 0 {
                            res.ast = alts[0].clone();
                            return res;
                        }

                        for x in arr {
                            alts.push(x.get("sequence").unwrap());
                        }

                        res.ast = json!({ "Alternatives": alts });
                        res
                    }),
                },
                Rule::new(
                    "Action",
                    Sequence::new(
                        vec![
                            "=>".into(),
                            (
                                "value",
                                Pattern::Alternatives(vec![rule_ref("Throw"), rule_ref("Return")]),
                            )
                                .into(),
                        ],
                        Action::Return(reference("value")),
                    ),
                )
                .into(),
                Rule::new(
                    "Return",
                    Sequence::new(
                        vec![("value", rule_ref("Expression")).into()],
                        Action::Return(json!({"Return": reference("value")})),
                    ),
                )
                .into(),
                Rule::new(
                    "Throw",
                    Sequence::new(
                        vec!["throw".into(), ("error", rule_ref("Expression")).into()],
                        Action::Return(json!({"Throw": reference("error")})),
                    ),
                )
                .into(),
                RuleWithAction::new(
                    Rule::new(
                        "Sequence",
                        vec![
                            ("patterns", Repeat::once_or_more(rule_ref("Repeat")).into()).into(),
                            ("action", Repeat::at_most_once(rule_ref("Action")).into()).into(),
                        ],
                    ),
                    |at, mut res, context| {
                        res = transparent_ast(at, res, context);

                        let action = res.ast["action"].clone();
                        let patterns = res.ast.get_mut("patterns").unwrap().as_array_mut().unwrap();
                        if action.is_null() {
                            if patterns.len() == 1 {
                                res.ast = patterns.pop().unwrap();
                            } else {
                                res.ast = patterns.clone().into();
                            }
                        } else {
                            res.ast = json!({"Sequence": res.ast});
                        }
                        res
                    },
                ),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Repeat".to_string(),
                        pattern: vec![
                            (
                                "pattern",
                                Pattern::RuleReference("AtomicPattern".to_string()),
                            )
                                .into(),
                            (
                                "op",
                                Repeat::at_most_once(Pattern::Alternatives(
                                    vec!["?".into(), "+".into(), "*".into()].into(),
                                ))
                                .into(),
                            )
                                .into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast(at, res, context);

                        let pattern = res.ast.get_mut("pattern").unwrap().take();
                        let op = res.ast.get_mut("op").unwrap().take();
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
                            Pattern::RuleReference("Named".to_string()),
                            Pattern::RuleReference("RuleReference".to_string()),
                            Pattern::RuleReference("Regex".to_string()),
                            Pattern::RuleReference("Text".to_string()),
                        ]),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                Rule {
                    name: "Named".to_string(),
                    pattern: vec![
                        "<".into(),
                        ("name", Pattern::RuleReference("Identifier".to_string())).into(),
                        ":".into(),
                        ("pattern", Pattern::RuleReference("Pattern".to_string())).into(),
                        ">".into(),
                    ]
                    .into(),
                }
                .into(),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Identifier".to_string(),
                        pattern: r"/[a-zA-Z_][a-zA-Z0-9_]*/".into(),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "EmptyObject".to_string(),
                        pattern: vec!['{'.into(), '}'.into()].into(),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "NonEmptyObject".to_string(),
                        pattern: vec![
                            '{'.into(),
                            (
                                "initializers",
                                vec![
                                    ("head", Pattern::RuleReference("Initializer".to_string()))
                                        .into(),
                                    (
                                        "tail",
                                        Repeat::zero_or_more(vec![
                                            ','.into(),
                                            (
                                                "init",
                                                Pattern::RuleReference("Initializer".to_string()),
                                            )
                                                .into(),
                                        ])
                                        .into(),
                                    )
                                        .into(),
                                    Repeat::at_most_once(',').into(),
                                ]
                                .into(),
                            )
                                .into(),
                            '}'.into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast(at, res, context);

                        let arr = res.ast.get("initializers").unwrap();

                        let mut inits = arr.get("head").unwrap().as_object().unwrap().clone();
                        for init in arr.get("tail").unwrap().as_array().unwrap() {
                            inits.extend(init.get("init").unwrap().as_object().unwrap().clone())
                        }
                        res.ast = json!(inits);
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Object".to_string(),
                        pattern: Pattern::Alternatives(vec![
                            Pattern::RuleReference("EmptyObject".to_string()),
                            Pattern::RuleReference("NonEmptyObject".to_string()),
                        ]),
                    }),
                    on_parsed: Some(transparent_ast),
                },
                Rule::new(
                    "Expression",
                    Sequence::new(
                        vec![(
                            "expr",
                            Pattern::Alternatives(vec![
                                rule_ref("Object"),
                                rule_ref("String"),
                                rule_ref("Char"),
                                rule_ref("Integer"),
                                rule_ref("Variable"),
                            ]),
                        )
                            .into()],
                        Action::Return(reference("expr")),
                    ),
                )
                .into(),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Initializer".to_string(),
                        pattern: vec![
                            ("name", Pattern::RuleReference("Identifier".to_string())).into(),
                            ":".into(),
                            ("value", Pattern::RuleReference("Expression".to_string())).into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast(at, res, context);
                        res.ast = json!({ res.ast.get("name").unwrap().as_str().unwrap(): res.ast.get_mut("value").unwrap().take() });
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "PatternInParentheses".to_string(),
                        pattern: vec![
                            "(".into(),
                            ("pattern", Pattern::RuleReference("Pattern".to_string())).into(),
                            ")".into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|_at, mut res, _| {
                        res.ast = res
                            .ast
                            .get_mut("PatternInParentheses")
                            .unwrap()
                            .get_mut("pattern")
                            .unwrap()
                            .take();
                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Rule".to_string(),
                        pattern: vec![
                            ("name", Pattern::RuleReference("RuleName".to_string())).into(),
                            ":".into(),
                            ("pattern", Pattern::RuleReference("Pattern".to_string())).into(),
                        ]
                        .into(),
                    }),
                    on_parsed: Some(|at, mut res, context| {
                        res = transparent_ast(at, res, context);
                        let rule: Rule = serde_json::from_value(res.ast.clone()).unwrap();
                        context.root = context
                            .root
                            .clone()
                            .or(Pattern::RuleReference(rule.name.clone()));
                        context.add_rule(rule);
                        res
                    }),
                },
                Rule {
                    name: "Variable".to_string(),
                    pattern: Pattern::RuleReference("Identifier".to_string()),
                }
                .into(),
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
        errors::Expected,
        parsers::{ParseResult, Parser},
        Context, ParseTree, Pattern, Rule,
    };

    #[test]
    fn action() {
        let mut context = Context::default();
        let r = context.find_rule("Action").unwrap();
        assert_eq!(r.name, "Action");
        assert_eq!(r.parse("=> 'x'", &mut context).ast, json!({"Return": 'x'}));
        assert_eq!(
            r.parse("=> throw x", &mut context).ast,
            json!({
                "Throw": {
                        "Variable": "x"
                }
            })
        );
    }

    #[test]
    fn throw() {
        let mut context = Context::default();
        let r = context.find_rule("Throw").unwrap();
        assert_eq!(r.name, "Throw");
        assert_eq!(
            r.parse("throw 'x'", &mut context).ast,
            json!({"Throw": 'x'})
        );
    }

    #[test]
    fn ret() {
        let mut context = Context::default();
        let r = context.find_rule("Return").unwrap();
        assert_eq!(r.name, "Return");
        assert_eq!(r.parse("'x'", &mut context).ast, json!({"Return": 'x'}));
    }

    #[test]
    fn object() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Object").unwrap();
        assert_eq!(r.name, "Object");
        assert_eq!(r.parse("{}", &mut ctx).ast, json!({}));
        assert_eq!(r.parse("{x: \"x\"}", &mut ctx).ast, json!({"x": "x"}));
        assert_eq!(
            r.parse("{x: 'x', y: {},}", &mut ctx).ast,
            json!({'x': 'x', 'y': {}})
        );
    }

    #[test]
    fn expression() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Expression").unwrap();
        assert_eq!(r.name, "Expression");
        assert_eq!(r.parse("{}", &mut ctx).ast, json!({}));
        assert_eq!(r.parse("'('", &mut ctx).ast, json!('('));
        assert_eq!(r.parse("\"()\"", &mut ctx).ast, json!("()"));
        assert_eq!(r.parse("x", &mut ctx).ast, json!({ "Variable": "x" }));
        assert_eq!(r.parse("123", &mut ctx).ast, json!(123));
    }

    #[test]
    fn integer() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Integer").unwrap();
        assert_eq!(r.name, "Integer");
        assert_eq!(r.parse("123", &mut ctx).ast, json!(123));

        let big_integer = "99999999999999999999999999999999";
        assert_eq!(
            r.parse(big_integer, &mut ctx).ast,
            json!({ "Integer": big_integer })
        );
    }

    #[test]
    fn variable() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Variable").unwrap();
        assert_eq!(r.name, "Variable");
        assert_eq!(r.parse("x", &mut ctx).ast, json!({ "Variable": "x" }));
    }

    #[test]
    fn initializer() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Initializer").unwrap();
        assert_eq!(r.name, "Initializer");
        assert_eq!(r.parse("x: 'x'", &mut ctx).ast, json!({"x": "x"}));
    }

    #[test]
    fn char() {
        let mut ctx = Context::default();
        let rule_name = ctx.find_rule("Char").unwrap();
        assert_eq!(rule_name.name, "Char");
        assert_eq!(rule_name.parse("'x'", &mut ctx).ast, json!("x"));
    }

    #[test]
    fn string() {
        let mut ctx = Context::default();
        let rule_name = ctx.find_rule("String").unwrap();
        assert_eq!(rule_name.name, "String");
        assert_eq!(rule_name.parse("\"abc\"", &mut ctx).ast, json!("abc"));
    }

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
                ast: json!("Foo")
            }
        );
        assert_eq!(
            rule_name.parse("foo", &mut ctx),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("RuleName").with(Expected {
                    expected: "[A-Z][a-zA-Z0-9]*".into(),
                    at: 0
                }),
                ast: json!(null)
            }
        );
        assert_eq!(
            rule_name.parse("", &mut ctx),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("RuleName").with(Expected {
                    expected: "[A-Z][a-zA-Z0-9]*".into(),
                    at: 0
                }),
                ast: json!(null)
            }
        );
    }

    #[test]
    fn identifier() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Identifier").unwrap();
        assert_eq!(r.name, "Identifier");
        assert_eq!(r.parse("Foo", &mut ctx).ast, json!("Foo"));
        assert_eq!(r.parse("foo", &mut ctx).ast, json!("foo"));
    }

    #[test]
    fn named_pattern() {
        let mut ctx = Context::default();
        let r = ctx.find_rule("Named").unwrap();
        assert_eq!(r.name, "Named");

        assert_eq!(
            r.parse("<name: /[a-z]+/>", &mut ctx).ast,
            json!({"Named": { "name": "name", "pattern": "/[a-z]+/" }})
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
        );
        assert_eq!(
            r.parse("'(' <l: /[a-z]/> ')' => l", &mut context).ast,
            json!({"Sequence": {
                "patterns": [
                    '(',
                    {
                        "Named": {
                            "name": "l",
                            "pattern": "/[a-z]/"
                        }
                    },
                    ')'
                ],
                "action": {
                    "Return": {
                        "Variable": "l"
                    }
                }
            }})
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

    #[test]
    fn root() {
        let mut context = Context::default();

        assert_eq!(context.root, Pattern::RuleReference("Rule".to_string()));

        let root = context.root.clone();
        root.parse("Lol: kek", &mut context);
        assert_eq!(
            context.root,
            Pattern::Alternatives(vec![
                Pattern::RuleReference("Rule".to_string()),
                Pattern::RuleReference("Lol".to_string()),
            ])
        );

        let root = context.root.clone();
        assert_eq!(root.parse("kek", &mut context).ast, json!({ "Lol": "kek" }));
    }
}
