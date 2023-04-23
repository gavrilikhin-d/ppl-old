#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(assert_matches)]
#![feature(is_some_and)]

use std::{any::Any, error::Error};

use derive_more::From;
use nom::{
    self,
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_while1,
    character::complete::{alpha0, char, one_of, satisfy, space0},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::delimited,
    IResult, Parser,
};
use regex::Regex;

/// Parse tree
#[derive(Debug, PartialEq, Clone, From)]
pub enum ParseTree<'s> {
    /// Token
    Token(&'s str),
    /// Tree with children
    Tree(Vec<ParseTree<'s>>),
}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(v: Vec<&'s str>) -> Self {
        Self::Tree(v.into_iter().map(|s| s.into()).collect())
    }
}

impl ParseTree<'_> {
    /// Append another tree to this tree
    pub fn append(&mut self, tree: impl Into<Self>) -> &mut Self {
        let tree = tree.into();
        match self {
            Self::Token(_) => *self = Self::Tree(vec![self.clone(), tree]),
            Self::Tree(v) => v.push(tree),
        };
        self
    }
}

/// [A-Z]
pub fn uppercase_alpha(input: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_uppercase())(input)
}

/// RuleName: [A-Z][a-zA-Z]*
pub fn rule_name(input: &str) -> IResult<&str, &str> {
    let (new_input, _) = uppercase_alpha(input)?;
    let (new_input, tail) = alpha0(new_input)?;
    Ok((new_input, &input[..1 + tail.len()]))
}

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule<'s> {
    /// Rule name
    pub name: &'s str,
    /// Rule patterns
    pub patterns: Vec<Pattern<'s>>,
}

/// Rule: RuleName: Pattern+
pub fn rule(input: &str) -> IResult<&str, (ParseTree, Rule)> {
    let (rest, name) = rule_name(input)?;
    let mut tree: ParseTree = name.into();

    let (rest, colon) = delimited(space0, tag(":"), space0)(rest)?;
    tree.append(colon);

    let (rest, v) = many1(delimited(space0, pattern, space0))(rest)?;
    v.iter().for_each(|(s, _)| {
        tree.append(*s);
    });

    Ok((
        rest,
        (
            tree,
            Rule {
                name,
                patterns: v.into_iter().map(|(_, p)| p).collect(),
            },
        ),
    ))
}

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
                    return Ok((
                        &input[m.end()..],
                        (ParseTree::from(m.as_str()), Box::new(m.as_str().to_owned())),
                    ));
                } else {
                    return err!(RegexMismatch {});
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
                return last_result.unwrap();
            }
            _ => unimplemented!(),
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

/// Pattern: Repeat | Alternatives
pub fn pattern(input: &str) -> IResult<&str, (&str, Pattern)> {
    alt((map(repeat, |(s, r)| (s, r.into())), alternatives))(input)
}

/// Repeat: BasicPattern ('+' | '*' | '?')
pub fn repeat(input: &str) -> IResult<&str, (&str, Repeat)> {
    let (rest, (_, p)) = basic_pattern(input)?;
    let (rest, c) = one_of("+*?")(rest)?;
    Ok((
        rest,
        (
            &input[..input.len() - rest.len()],
            match c {
                '*' => Repeat::zero_or_more(p),
                '+' => Repeat::once_or_more(p),
                '?' => Repeat::at_most_once(p),
                _ => unreachable!(),
            },
        ),
    ))
}

/// Alternatives: BasicPattern ( '|' BasicPattern )*
pub fn alternatives(input: &str) -> IResult<&str, (&str, Pattern)> {
    let (rest, v) = separated_list1(delimited(space0, char('|'), space0), basic_pattern)(input)?;
    Ok((
        rest,
        if v.len() == 1 {
            v.into_iter().next().unwrap()
        } else {
            (
                &input[..input.len() - rest.len()],
                Pattern::Alternatives(v.into_iter().map(|(_, p)| p).collect()),
            )
        },
    ))
}

/// BasicPattern: RuleReference | Group | Regex
pub fn basic_pattern(input: &str) -> IResult<&str, (&str, Pattern)> {
    alt((
        map(rule_reference, |s| (s, Pattern::RuleReference(s))),
        map(group, |(s, v)| {
            (
                s,
                if v.len() == 1 {
                    v.into_iter().next().unwrap()
                } else {
                    Pattern::Group(v)
                },
            )
        }),
        map(regex, |s| (s, Pattern::Regex(s))),
    ))(input)
}

/// RuleReference: RuleName
pub fn rule_reference(input: &str) -> IResult<&str, &str> {
    rule_name(input)
}

/// Group: '(' Pattern+ ')'
pub fn group(input: &str) -> IResult<&str, (&str, Vec<Pattern>)> {
    let (rest, v) = delimited(
        char('('),
        many1(delimited(space0, pattern, space0)),
        char(')'),
    )(input)?;
    Ok((
        rest,
        (
            &input[..input.len() - rest.len()],
            v.into_iter().map(|(_, p)| p).collect(),
        ),
    ))
}

/// Regex: [^ \t\r\n()|]+
pub fn regex(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !(c.is_whitespace() || ['(', ')', '|'].contains(&c)))(input)
}

#[cfg(test)]
mod test {
    use nom::Parser;

    use crate::{
        alternatives, basic_pattern, regex, repeat, rule, rule_name, ParseTree, Pattern, Repeat,
        Rule,
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
            Ok(("", ("(x)+", Repeat::once_or_more(p.clone()).into())))
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
    }
}
