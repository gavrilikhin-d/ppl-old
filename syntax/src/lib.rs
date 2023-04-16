#![feature(anonymous_lifetime_in_impl_trait)]

use nom::{
    self,
    character::complete::{alpha0, satisfy},
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

#[cfg(test)]
mod test {
    use crate::rule;

    #[test]
    fn test_rule() {
        assert_eq!(rule("ValidRuleName"), Ok(("", "ValidRuleName")));
        assert!(rule("invalidRuleName").is_err());
    }
}
