use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::{parsers, ParseTree, Pattern};

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Rule patterns
    pub patterns: Vec<Pattern>,
}

impl<'i> Parser<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> for Rule {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> {
        let (r, (t, ast)) = parsers::grouped_patterns(&mut self.patterns, input)?;
        Ok((r, (t, Box::new(ast))))
    }
}
