use crate::{
    parsers::{ParseResult, Parser},
    ParseTree,
};

use super::Pattern;

/// Repeat pattern
#[derive(Debug, PartialEq, Clone)]
pub struct Repeat {
    /// Pattern to repeat
    pub pattern: Box<Pattern>,
    /// Minimum number of repetitions
    pub at_least: usize,
    /// Maximum number of repetitions
    pub at_most: Option<usize>,
}

impl Repeat {
    /// Repeat pattern zero or more times (x*)
    pub fn zero_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: None,
        }
    }

    /// Repeat pattern once or more times (x+)
    pub fn once_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 1,
            at_most: None,
        }
    }

    /// Repeat pattern at most once (x?)
    pub fn at_most_once(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: Some(1),
        }
    }
}

impl Parser for Repeat {
    fn parse_at<'s>(&self, source: &'s str, at: usize) -> ParseResult<'s> {
        debug_assert!(self.at_least <= self.at_most.unwrap_or(usize::MAX));

        let mut delta = 0;
        let mut tree = ParseTree::Tree(vec![]);
        for _ in 0..self.at_least {
            let res = self.pattern.parse_at(source, at + delta);
            delta += res.delta;
            tree.append(res.tree);
        }

        if tree.is_ok() {
            for _ in self.at_least..self.at_most.unwrap_or(usize::MAX) {
                let res = self.pattern.parse_at(source, at + delta);
                if res.is_ok() {
                    delta += res.delta;
                    tree.append(res.tree);
                } else {
                    break;
                }
            }
        }

        ParseResult { delta, tree }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        errors::Expected,
        parsers::{ParseResult, Parser},
        IntoParseTree, ParseTree,
    };

    use super::Repeat;

    #[test]
    fn at_most_once() {
        let pattern = Repeat::at_most_once("a".into());
        assert_eq!(
            pattern.parse_at("", 0),
            ParseResult {
                delta: 0,
                tree: ParseTree::Tree(vec![]),
            }
        );
        assert_eq!(
            pattern.parse_at("a", 0),
            ParseResult {
                delta: 1,
                tree: vec!["a"].into(),
            }
        );
        assert_eq!(
            pattern.parse_at("aa", 0),
            ParseResult {
                delta: 1,
                tree: vec!["a"].into(),
            }
        )
    }

    #[test]
    fn zero_or_more() {
        let pattern = Repeat::zero_or_more("a".into());
        assert_eq!(
            pattern.parse_at("", 0),
            ParseResult {
                delta: 0,
                tree: ParseTree::Tree(vec![]),
            }
        );
        assert_eq!(
            pattern.parse_at("a", 0),
            ParseResult {
                delta: 1,
                tree: vec!["a"].into(),
            }
        );
        assert_eq!(
            pattern.parse_at("aa", 0),
            ParseResult {
                delta: 2,
                tree: vec!["a", "a"].into(),
            }
        );
    }

    #[test]
    fn once_or_more() {
        let pattern = Repeat::once_or_more("a".into());
        assert_eq!(
            pattern.parse_at("", 0),
            ParseResult {
                delta: 0,
                tree: vec![Expected {
                    expected: "a".to_string(),
                    at: 0.into()
                }
                .into_parse_tree()]
                .into(),
            }
        );
        assert_eq!(
            pattern.parse_at("a", 0),
            ParseResult {
                delta: 1,
                tree: vec!["a"].into(),
            }
        );
        assert_eq!(
            pattern.parse_at("aa", 0),
            ParseResult {
                delta: 2,
                tree: vec!["a", "a"].into(),
            }
        );
    }
}
