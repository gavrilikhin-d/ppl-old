use derive_more::From;
use regex::Regex;

use crate::{error::UnexpectedToken, CaptureMatch, MatchError, Parser, PatternMatch, RegexMatch};

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
        start: usize,
        parser: &Parser,
    ) -> Result<PatternMatch<'source>, MatchError<'source>> {
        match self {
            Pattern::Regex(regex) => {
                if let Some(m) = regex.find(&source[start..]) {
                    Ok(RegexMatch {
                        source,
                        start,
                        end: start + m.range().len(),
                    }
                    .into())
                } else {
                    let end = start
                        + source[start..]
                            .find(char::is_whitespace)
                            .unwrap_or(source.len() - start);
                    Err(MatchError {
                        source,
                        start,
                        end,
                        payload: UnexpectedToken {
                            expected: regex.to_string(),
                            got: source[start..end].to_string(),
                        }
                        .into(),
                    })
                }
            }
            Pattern::Rule(rule) => Ok(parser
                .try_rule(rule)
                .map_err(|e| MatchError {
                    source,
                    start,
                    end: start,
                    payload: e.into(),
                })?
                .apply(source, start, parser)?
                .into()),
            Pattern::Capture { name, pattern } => {
                if let Ok(m) = pattern.apply(source, start, parser) {
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
        Ok(Pattern::Regex(Regex::new(value)?))
    }
}
