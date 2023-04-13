use std::rc::Rc;

use derive_more::From;
use regex::Regex;

mod alternatives;
pub use alternatives::*;

mod group;
pub use group::*;

mod repeat;
pub use repeat::*;

use crate::{
    error::{UnexpectedEOF, UnexpectedToken},
    Parser, PatternMatch, Rule, SubsliceOffset,
};

/// Syntax pattern
#[derive(From)]
pub enum Pattern {
    /// Match token using regex
    Regex(Regex),
    /// Reference to another rule
    Rule(Rc<Rule>),
    /// Group multiple patterns
    Group(Group),
    /// Repeat pattern
    Repeat(Repeat),
    /// Match one of the alternatives
    Alternatives(Alternatives),
}

impl Pattern {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut (impl Iterator<Item = &'source str> + Clone),
        parser: &mut Parser,
    ) -> PatternMatch<'source> {
        match self {
            Pattern::Regex(regex) => {
                let token = tokens.next();
                if token.is_none() {
                    return UnexpectedEOF {
                        expected: regex.to_string(),
                        at: source.len(),
                    }
                    .into();
                }
                let token = token.unwrap();

                if regex.is_match(token) {
                    token.into()
                } else {
                    UnexpectedToken {
                        expected: regex.to_string(),
                        got: token.into(),
                        at: token
                            .offset_in(source)
                            .expect("Token isn't a subslice of source"),
                    }
                    .into()
                }
            }
            Pattern::Rule(rule) => rule.apply(source, tokens, parser).into(),
            Pattern::Group(group) => group.apply(source, tokens, parser).into(),
            Pattern::Repeat(rep) => rep.apply(source, tokens, parser).into(),
            Pattern::Alternatives(a) => a.apply(source, tokens, parser).into(),
        }
    }
}

impl TryFrom<&str> for Pattern {
    type Error = regex::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Pattern::Regex(Regex::new(&format!("^{value}$"))?))
    }
}
