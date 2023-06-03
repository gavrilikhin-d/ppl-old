mod named;
pub use named::*;

mod repeat;
pub use repeat::*;

mod sequence;
pub use sequence::*;

mod rule_ref;
pub use rule_ref::*;

use derive_more::From;

use regex::Regex;

use serde::{Deserialize, Serialize};

use crate::{
    action::{reference, ret, Expression},
    bootstrap::rules::Alternatives,
    errors::Expected,
    parsers::{ParseResult, Parser},
    rule_ref, Context, ParseTreeNode, Rule, Token,
};

/// Possible patterns
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Pattern {
    /// Reference to another rule
    RuleReference(Box<RuleReference>),
    /// Sequence of patterns
    Sequence(Sequence),
    /// Match specific text
    #[from(ignore)]
    Text(String),
    /// Regex expression
    #[from(ignore)]
    Regex(String),
    /// Pattern alternatives
    #[from(ignore)]
    Alternatives(Vec<Pattern>),
    /// Repeat pattern
    Repeat(Repeat),
    /// Adds name to the ast of pattern
    Named(Named),
}

/// <head: Pattern> <tail: (Separator (<value: Pattern> => value))*> => [head, ...tail]
pub fn separated(pattern: impl Into<Pattern>, separator: impl Into<Pattern>) -> Pattern {
    let pattern = pattern.into();
    Sequence::new(
        vec![
            ("head", pattern.clone()).into(),
            (
                "tail",
                Repeat::zero_or_more(Sequence::new(
                    vec![separator.into(), ("value", pattern).into()],
                    ret(reference("value")),
                ))
                .into(),
            )
                .into(),
        ],
        ret(Expression::Flatten(vec![
            reference("head"),
            reference("tail"),
        ])),
    )
    .into()
}

#[macro_export]
macro_rules! alts {
    ($head: expr, $($tail: expr),+) => {
		crate::Pattern::Alternatives(vec![$head.into(), $($tail.into()),+].into())
	};
}

impl Pattern {
    /// Return an alternative pattern between this pattern and another
    pub fn or(mut self, other: Pattern) -> Self {
        match &mut self {
            Pattern::Alternatives(alts) => alts.push(other),
            _ => {
                self = Pattern::Alternatives(vec![self, other]);
            }
        }
        self
    }

    pub fn rule() -> Rule {
        Rule::new("Pattern", transparent(rule_ref!(Alternatives)))
    }
}

impl Serialize for Pattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PatternDTO::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto = PatternDTO::deserialize(deserializer)?;
        Ok(dto.into())
    }
}

#[derive(Serialize, Deserialize, From)]
enum PatternDTO {
    RuleReference(Box<RuleReference>),
    #[from(ignore)]
    Alternatives(Vec<Pattern>),
    Repeat(Repeat),
    Named(Named),
    Sequence(Sequence),

    #[serde(untagged)]
    TextOrRegex(String),
    #[serde(untagged)]
    SequenceWithoutAction(Vec<Pattern>),
}

impl From<Pattern> for PatternDTO {
    fn from(value: Pattern) -> Self {
        match value {
            Pattern::Text(t) => PatternDTO::TextOrRegex(t),
            Pattern::Regex(r) => PatternDTO::TextOrRegex(format!("/{}/", r)),
            Pattern::Sequence(s) => {
                if s.action.is_none() {
                    PatternDTO::SequenceWithoutAction(s.patterns)
                } else {
                    PatternDTO::Sequence(s).into()
                }
            }

            Pattern::RuleReference(r) => PatternDTO::RuleReference(r).into(),
            Pattern::Named(named) => PatternDTO::Named(named).into(),
            Pattern::Repeat(r) => PatternDTO::Repeat(r).into(),
            Pattern::Alternatives(alts) => {
                PatternDTO::Alternatives(alts.into_iter().map(|a| a.into()).collect()).into()
            }
        }
    }
}

impl From<PatternDTO> for Pattern {
    fn from(value: PatternDTO) -> Self {
        match value {
            PatternDTO::TextOrRegex(t) => t.into(),
            PatternDTO::Sequence(s) => s.into(),
            PatternDTO::RuleReference(r) => r.into(),
            PatternDTO::Named(n) => n.into(),
            PatternDTO::Repeat(r) => r.into(),
            PatternDTO::Alternatives(alts) => Pattern::Alternatives(alts),
            PatternDTO::SequenceWithoutAction(s) => s.into(),
        }
    }
}

impl From<char> for Pattern {
    fn from(value: char) -> Self {
        Pattern::Text(value.to_string())
    }
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        if s.len() > 1 && s.starts_with('/') && s.ends_with('/') {
            return Pattern::Regex(s[1..s.len() - 1].to_string());
        }
        Pattern::Text(s.into())
    }
}

impl From<String> for Pattern {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<(&str, Pattern)> for Pattern {
    fn from(value: (&str, Pattern)) -> Self {
        Pattern::Named(Named {
            name: value.0.into(),
            pattern: Box::new(value.1),
        })
    }
}

impl From<Vec<Pattern>> for Pattern {
    fn from(value: Vec<Pattern>) -> Self {
        Pattern::Sequence(value.into())
    }
}

impl Parser for Pattern {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        match self {
            Pattern::Text(text) => {
                Pattern::Regex(regex::escape(text)).parse_at(source, at, context)
            }
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
            Pattern::RuleReference(r) => r.parse_at(source, at, context),
            Pattern::Sequence(s) => s.parse_at(source, at, context),
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

    use pretty_assertions::assert_eq;

    use crate::{
        errors::Expected,
        parsers::{ParseResult, Parser},
        patterns::Named,
        Context, ParseTree, ParseTreeNode, Pattern,
    };

    #[test]
    fn text() {
        let mut context = Context::default();
        let pattern: Pattern = "()".into();
        assert_eq!(pattern, Pattern::Text("()".into()));
        assert_eq!(
            pattern.parse("()", &mut context),
            ParseResult {
                delta: 2,
                tree: "()".into(),
                ast: json!("()")
            }
        );
    }

    #[test]
    fn regex() {
        let mut context = Context::default();
        let pattern: Pattern = r"/[^\s]+/".into();
        assert_eq!(pattern, Pattern::Regex(r"[^\s]+".into()));
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
    fn sequence() {
        let mut context = Context::default();
        let pattern = Pattern::Sequence(vec!["a".into(), "b".into()].into());
        assert_eq!(
            pattern.parse("ab", &mut context),
            ParseResult {
                delta: 2,
                tree: vec!["a", "b"].into(),
                ast: json!({})
            }
        );
        assert_eq!(
            pattern.parse("b", &mut context),
            ParseResult {
                delta: 0,
                tree: vec![ParseTreeNode::from(Expected {
                    expected: "a".to_string(),
                    at: 0
                }),]
                .into(),
                ast: json!(null)
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
                ast: json!(null)
            }
        );
        assert_eq!(
            pattern.parse("", &mut context),
            ParseResult {
                delta: 0,
                tree: vec![ParseTreeNode::from(Expected {
                    expected: "a".to_string(),
                    at: 0
                })]
                .into(),
                ast: json!(null)
            }
        )
    }

    #[test]
    fn rule_ref() {
        let mut context = Context::default();
        let pattern = crate::rule_ref!("Text");
        assert_eq!(
            pattern.parse("abc", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Text").with("abc"),
                ast: json!("abc")
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
            pattern: Box::new("/[A-z][a-z]*/".into()),
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
