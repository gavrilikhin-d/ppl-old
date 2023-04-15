#![feature(anonymous_lifetime_in_impl_trait)]

use nom::{
    self,
    character::complete::{alpha0, one_of},
    IResult,
};

/// [A-Z]
pub fn uppercase_alpha(input: &str) -> IResult<&str, char> {
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(&input)
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
    #[test]
    fn rule() {
        assert_eq!(super::rule("ValidRuleName"), Ok(("", "ValidRuleName")));
        assert!(super::rule("invalidRuleName").is_err());
    }
}
