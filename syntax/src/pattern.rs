use derive_more::From;
use regex::Regex;

use crate::{error::UnexpectedToken, CaptureMatch, Error, Parser, PatternMatch, SubsliceOffset};

/// Syntax pattern
#[derive(Debug, From)]
pub enum Pattern {
    /// Match token using regex
    Regex(Regex),
    /// Reference to another rule
    Rule(String),
    /// Capture a pattern
    Capture {
        /// Name of the capture
        name: String,
        /// Pattern to capture
        pattern: Box<Pattern>,
    },
}

impl Pattern {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut impl Iterator<Item = &'source str>,
        parser: &Parser,
    ) -> Result<PatternMatch<'source>, Error> {
        match self {
            Pattern::Regex(regex) => {
                let token = tokens.next();
                if token.is_none() {
                    unimplemented!("error")
                }
                let token = token.unwrap();

                if regex.is_match(token) {
                    Ok(token.into())
                } else {
                    Err(UnexpectedToken {
                        expected: regex.to_string(),
                        got: token.into(),
                        at: token
                            .offset_in(source)
                            .expect("Token isn't a subslice of source"),
                    }
                    .into())
                }
            }
            Pattern::Rule(rule) => Ok(parser.try_rule(rule)?.apply(source, tokens, parser)?.into()),
            Pattern::Capture { name, pattern } => {
                if let Ok(m) = pattern.apply(source, tokens, parser) {
                    Ok(CaptureMatch {
                        name: name.clone(),
                        matched: Box::new(m),
                    }
                    .into())
                } else {
                    unimplemented!("error")
                }
            }
        }
    }
}

impl TryFrom<&str> for Pattern {
    type Error = regex::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Pattern::Regex(Regex::new(&format!("^{value}$"))?))
    }
}
