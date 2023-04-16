#![feature(anonymous_lifetime_in_impl_trait)]

use nom::{
    self,
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alpha0, satisfy},
    combinator::map,
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
#[derive(Debug, PartialEq)]
pub enum Pattern<'s> {
    /// Reference to another rule
    RuleReference(&'s str),
    /// Regex
    Regex(&'s str),
}

/// Pattern: RuleReference | Regex
pub fn pattern(input: &str) -> IResult<&str, (&str, Pattern)> {
    alt((
        map(rule_reference, |s| (s, Pattern::RuleReference(s))),
        map(regex, |s| (s, Pattern::Regex(s))),
    ))(input)
}

/// RuleReference: RuleName
pub fn rule_reference(input: &str) -> IResult<&str, &str> {
    rule_name(input)
}

/// Regex: [^ \t\r\n]+
pub fn regex(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(input)
}

#[cfg(test)]
mod test {
    use crate::{pattern, regex, rule_name, Pattern};

    #[test]
    fn test_rule_name() {
        assert_eq!(rule_name("ValidRuleName"), Ok(("", "ValidRuleName")));
        assert!(rule_name("invalidRuleName").is_err());
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
    fn test_pattern() {
        assert_eq!(
            pattern("ValidRuleName"),
            Ok((
                "",
                ("ValidRuleName", Pattern::RuleReference("ValidRuleName"))
            ))
        );
        assert_eq!(
            pattern("validRegex"),
            Ok(("", ("validRegex", Pattern::Regex("validRegex"))))
        );
    }
}
