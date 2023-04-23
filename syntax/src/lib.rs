#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(assert_matches)]
#![feature(is_some_and)]

use std::{any::Any, error::Error};

mod tree;
pub use tree::*;

pub mod patterns;

mod rule;
pub use rule::*;

pub mod parsers;

use derive_more::From;
use nom::{self, IResult, Parser};
use regex::Regex;

#[derive(Debug, thiserror::Error)]
#[error("Regex didn't match")]
pub struct RegexMismatch {}

/// Possible patterns
#[derive(Debug, PartialEq, Clone, From)]
pub enum Pattern<'s> {
    /// Reference to another rule
    RuleReference(&'s str),
    /// Group of patterns
    Group(Vec<Pattern<'s>>),
    /// Regex
    Regex(&'s str),
    /// Pattern alternatives
    Alternatives(Vec<Pattern<'s>>),
    /// Repeat pattern
    #[from]
    Repeat(Repeat<'s>),
}

/// Creates recoverable error for nom
macro_rules! err {
    ($error: expr) => {
        Err(nom::Err::Error(Box::new($error)))
    };
}

impl<'i, 's> Parser<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> for Pattern<'s> {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> {
        match self {
            Self::Regex(r) => {
                let re = Regex::new(&format!("^{}", r)).unwrap();
                let m = re.find(input);
                if let Some(m) = m {
                    Ok((
                        &input[m.end()..],
                        (ParseTree::from(m.as_str()), Box::new(m.as_str().to_owned())),
                    ))
                } else {
                    err!(RegexMismatch {})
                }
            }
            Self::Alternatives(alts) => {
                let mut last_result = None;
                for alt in alts {
                    let res = alt.parse(input);
                    if res.is_ok()
                        || res
                            .as_ref()
                            .is_err_and(|e| matches!(e, nom::Err::Failure(_)))
                    {
                        return res;
                    }
                    last_result = Some(res);
                }
                last_result.unwrap()
            }
            Self::Repeat(r) => Ok({
                let (r, (t, ast)) = r.parse(input)?;
                (r, (t, Box::new(ast)))
            }),
            Self::Group(patterns) => {
                let (r, (t, ast)) = parsers::grouped_patterns(patterns, input)?;
                Ok((r, (t, Box::new(ast))))
            }
            Self::RuleReference(_) => unimplemented!(),
        }
    }
}

/// Repeat pattern
#[derive(Debug, PartialEq, Clone)]
pub struct Repeat<'s> {
    /// Pattern to repeat
    pub pattern: Box<Pattern<'s>>,
    /// Minimum number of repetitions
    pub at_least: usize,
    /// Maximum number of repetitions
    pub at_most: Option<usize>,
}

impl<'s> Repeat<'s> {
    /// Repeat pattern zero or more times (x*)
    pub fn zero_or_more(pattern: Pattern<'s>) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: None,
        }
    }

    /// Repeat pattern once or more times (x+)
    pub fn once_or_more(pattern: Pattern<'s>) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 1,
            at_most: None,
        }
    }

    /// Repeat pattern at most once (x?)
    pub fn at_most_once(pattern: Pattern<'s>) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: Some(1),
        }
    }
}

impl<'i, 's> Parser<&'i str, (ParseTree<'i>, Vec<Box<dyn Any>>), Box<dyn Error>> for Repeat<'s> {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Vec<Box<dyn Any>>), Box<dyn Error>> {
        debug_assert!(self.at_most.is_none() || self.at_most.unwrap() >= self.at_least);
        let mut input = input;
        let mut trees = Vec::new();
        let mut asts = Vec::new();
        for _ in 0..self.at_least {
            let (rest, (tree, ast)) = self.pattern.parse(input)?;
            input = rest;
            trees.push(tree);
            asts.push(ast);
        }

        for _ in self.at_least..self.at_most.unwrap_or(usize::MAX) {
            let res = self.pattern.parse(input);
            if res.is_ok() {
                let (rest, (tree, ast)) = res.unwrap();
                input = rest;
                trees.push(tree);
                asts.push(ast);
            } else {
                break;
            }
        }

        Ok((input, (trees.into(), asts)))
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;

    use nom::Parser;

    use crate::{
        parsers::{alternatives, basic_pattern, regex, repeat, rule, rule_name},
        ParseTree, Pattern, Repeat, Rule,
    };

    #[test]
    fn test_rule_name() {
        assert_eq!(rule_name("ValidRuleName"), Ok(("", "ValidRuleName")));
        assert!(rule_name("invalidRuleName").is_err());
    }

    #[test]
    fn test_group() {
        assert_eq!(
            basic_pattern("(Rule)"),
            Ok(("", ("(Rule)", Pattern::RuleReference("Rule"))))
        );
        assert_eq!(
            basic_pattern("(Rule | [a-z])"),
            Ok((
                "",
                (
                    "(Rule | [a-z])",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("Rule"),
                        Pattern::Regex("[a-z]")
                    ])
                )
            ))
        );
        assert_eq!(
            basic_pattern("(Rule [a-z])"),
            Ok((
                "",
                (
                    "(Rule [a-z])",
                    Pattern::Group(vec![
                        Pattern::RuleReference("Rule"),
                        Pattern::Regex("[a-z]")
                    ])
                )
            ))
        );
    }

    #[test]
    fn test_regex() {
        assert_eq!(regex("ValidRegex"), Ok(("", "ValidRegex")));
        assert_eq!(
            regex("Vali1324dRegex rest"),
            Ok((" rest", "Vali1324dRegex"))
        );

        assert_eq!(regex("x+"), Ok(("", "x+")));
        assert_eq!(regex("x*"), Ok(("", "x*")));
        assert_eq!(regex("x?"), Ok(("", "x?")));
    }

    #[test]
    fn test_basic_pattern() {
        assert_eq!(
            basic_pattern("ValidRuleName"),
            Ok((
                "",
                ("ValidRuleName", Pattern::RuleReference("ValidRuleName"))
            ))
        );
        assert_eq!(
            basic_pattern("validRegex"),
            Ok(("", ("validRegex", Pattern::Regex("validRegex"))))
        );
        assert_eq!(
            basic_pattern("(x y)"),
            Ok((
                "",
                (
                    "(x y)",
                    Pattern::Group(vec![Pattern::Regex("x"), Pattern::Regex("y")])
                )
            ))
        );
    }

    #[test]
    fn test_repeat() {
        let p = Pattern::Regex("x");
        assert_eq!(
            repeat("(x)+"),
            Ok(("", ("(x)+", Repeat::once_or_more(p.clone()))))
        );
        assert_eq!(
            repeat("(x)*"),
            Ok(("", ("(x)*", Repeat::zero_or_more(p.clone()))))
        );
        assert_eq!(
            repeat("(x)?"),
            Ok(("", ("(x)?", Repeat::at_most_once(p.clone()))))
        )
    }

    #[test]
    fn test_alternatives() {
        assert_eq!(
            alternatives("ValidRuleName | [a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName | [a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName"),
                        Pattern::Regex("[a-z]"),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName| [a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName| [a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName"),
                        Pattern::Regex("[a-z]"),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName |[a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName |[a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName"),
                        Pattern::Regex("[a-z]"),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName|[a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName|[a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName"),
                        Pattern::Regex("[a-z]"),
                    ])
                )
            ))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule("Rule: x"),
            Ok((
                "",
                (
                    ParseTree::from(vec!["Rule", ":", "x"]),
                    Rule {
                        name: "Rule",
                        patterns: vec![Pattern::Regex("x")]
                    }
                )
            ))
        )
    }

    #[test]
    fn test_pattern_as_parser() {
        let res = Pattern::Regex("x+").parse("xxx");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from("xxx"));
        assert_eq!(ast.downcast::<String>().ok().unwrap().as_str(), "xxx");

        let res = Pattern::Alternatives(vec![Pattern::Regex("x"), Pattern::Regex("y")]).parse("y");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from("y"));
        assert_eq!(ast.downcast::<String>().ok().unwrap().as_str(), "y");

        let res = Pattern::Group(vec![Pattern::Regex("x"), Pattern::Regex("y")]).parse("xy");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from(vec!["x", "y"]));
        assert_eq!(
            ast.downcast::<Vec<Box<dyn Any>>>()
                .ok()
                .unwrap()
                .into_iter()
                .map(|x| x.downcast::<String>().ok().unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
            &vec![Box::new("x".to_string()), Box::new("y".to_string())]
        );
    }

    #[test]
    fn test_repeat_as_parser() {
        let res = Repeat::at_most_once(Pattern::Regex("x")).parse("");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::Tree(vec![]));
        assert!(ast.is_empty());
    }

    #[test]
    fn test_rule_as_parser() {
        let res = Rule {
            name: "Rule",
            patterns: vec![Pattern::Regex("x")],
        }
        .parse("x");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from(vec!["x"]));
        assert_eq!(
            ast.downcast::<Vec<Box<dyn Any>>>()
                .ok()
                .unwrap()
                .into_iter()
                .map(|x| x.downcast::<String>().ok().unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
            &vec![Box::new("x".to_string())]
        );
    }
}
