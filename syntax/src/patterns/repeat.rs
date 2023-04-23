use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::ParseTree;

use super::Pattern;

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
