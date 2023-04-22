#![feature(anonymous_lifetime_in_impl_trait)]

use derive_more::From;
use nom::{
    self,
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alpha0, char, one_of, satisfy, space0},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::delimited,
    IResult,
};

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

/// Rule: RuleName
pub fn rule(input: &str) -> IResult<&str, &str> {
    rule_name(input)
}

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

/// Regex: [^ \t\r\n()+*?|]+
pub fn regex(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !(c.is_whitespace() || ['(', ')', '+', '*', '?', '|'].contains(&c)))(
        input,
    )
}

#[cfg(test)]
mod test {
    use crate::{alternatives, basic_pattern, pattern, regex, repeat, rule_name, Pattern, Repeat};

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
            repeat("x+"),
            Ok(("", ("x+", Repeat::once_or_more(p.clone()).into())))
        );
        assert_eq!(
            repeat("x*"),
            Ok(("", ("x*", Repeat::zero_or_more(p.clone()))))
        );
        assert_eq!(
            repeat("x?"),
            Ok(("", ("x?", Repeat::at_most_once(p.clone()))))
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
}
