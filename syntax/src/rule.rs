use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::{grouped_patterns, ParseTree, Pattern};

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule<'s> {
    /// Rule name
    pub name: &'s str,
    /// Rule patterns
    pub patterns: Vec<Pattern<'s>>,
}

impl<'i, 's> Parser<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> for Rule<'s> {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> {
        let (r, (t, ast)) = grouped_patterns(&mut self.patterns, input)?;
        Ok((r, (t, Box::new(ast))))
    }
}
