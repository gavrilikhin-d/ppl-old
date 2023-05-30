use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    action::Action,
    errors::Error,
    parsers::{ParseResult, Parser},
    Context, ParseTree, Pattern,
};

/// Sequence of patterns with optional action
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Sequence {
    /// Patterns to parse one after another
    pub patterns: Vec<Pattern>,
    /// Action to perform after parsing
    #[serde(default)]
    pub action: Option<Action>,
}

impl Sequence {
    /// Create a new sequence with an action
    pub fn new(patterns: Vec<Pattern>, action: Action) -> Self {
        Self {
            patterns,
            action: Some(action),
        }
    }
}

impl From<Vec<Pattern>> for Sequence {
    fn from(patterns: Vec<Pattern>) -> Self {
        Self {
            patterns,
            action: None,
        }
    }
}

impl Parser for Sequence {
    fn parse_at<'s>(
        &self,
        source: &'s str,
        at: usize,
        context: &mut Context,
    ) -> crate::parsers::ParseResult<'s> {
        let mut delta = 0;
        let mut tree = ParseTree::empty();
        let mut ast = json!({});
        for pattern in &self.patterns {
            let mut result = pattern.parse_at(source, at + delta, context);
            let has_errors = result.has_errors();
            delta += result.delta;
            tree.push(result.tree);

            if let Pattern::Named(_) = pattern {
                ast.as_object_mut()
                    .unwrap()
                    .append(&mut result.ast.as_object_mut().unwrap());
            } else if self.patterns.len() == 1 {
                ast = result.ast;
                break;
            }

            if has_errors {
                return ParseResult {
                    delta,
                    tree: tree.flatten(),
                    ast: json!(null),
                };
            }
        }

        if let Some(action) = &self.action {
            ast = expand_variables(
                action.value(),
                &ast.as_object().cloned().unwrap_or_default(),
            );
            if matches!(action, Action::Throw(_)) {
                let error: Error = serde_json::from_value(ast.clone()).unwrap();
                println!("{:?}", miette::Report::new(error));
                delta = 0;
            }
        }

        ParseResult {
            delta,
            tree: tree.flatten(),
            ast: ast.into(),
        }
    }
}

fn expand_variables(
    action: &serde_json::Value,
    ast: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    match action {
        serde_json::Value::Object(o) => {
            if o.keys().len() == 1 && o.keys().next().unwrap() == "Variable" {
                let variable = o.get("Variable").unwrap().as_str().unwrap();
                return ast.get(variable).unwrap().clone();
            }

            let mut result = serde_json::Map::new();
            for (key, value) in o {
                result.insert(key.clone(), expand_variables(value, ast));
            }
            result.into()
        }
        serde_json::Value::Array(a) => {
            let mut result = Vec::new();
            for value in a {
                result.push(expand_variables(value, ast));
            }
            result.into()
        }
        _ => action.clone(),
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{
        action::{reference, Action},
        parsers::Parser,
        Context, Pattern,
    };

    use super::Sequence;
    use pretty_assertions::assert_eq;

    #[test]
    fn named() {
        let mut context = Context::default();
        let p: Sequence = vec![
            '('.into(),
            ("pattern", "/[A-z][a-z]*/".into()).into(),
            ')'.into(),
        ]
        .into();

        assert_eq!(p.parse("( x )", &mut context).ast, json!({"pattern": "x"}));
    }

    #[test]
    fn action() {
        let mut context = Context::default();
        let p = Sequence::new(
            vec![
                '('.into(),
                ("pattern", "/[A-z][a-z]*/".into()).into(),
                ')'.into(),
            ],
            Action::Return(reference("pattern")),
        );

        assert_eq!(p.parse("( x )", &mut context).ast, json!("x"));
    }

    #[test]
    fn error() {
        let mut context = Context::default();
        let p = Pattern::Alternatives(vec![
            vec!['('.into(), "/[A-z][a-z]*/".into(), ')'.into()].into(),
            Sequence::new(
                vec!['('.into(), "/[A-z][a-z]*/".into()],
                Action::Throw(json!({ "Expected" : {
                    "expected": ")",
                    "at": 3
                }})),
            )
            .into(),
        ]);

        assert_eq!(p.parse("( x )", &mut context).ast, json!({}));

        assert_eq!(
            p.parse("( x", &mut context).ast,
            json!({
                "Expected": {
                    "expected": ")",
                    "at": 3
                }
            })
        );
    }
}
