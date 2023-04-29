use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::{context::with_context, err, parsers, ParseTree, Pattern};

/// Type of rule name
pub type RuleName = String;

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    /// Rule name
    pub name: RuleName,
    /// Rule patterns
    pub patterns: Vec<Pattern>,
}

impl<'i> Parser<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> for Rule {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> {
        let (r, (t, ast)) = parsers::grouped_patterns(&mut self.patterns, input)?;
        let ast = Box::new(ast);
        let action_res = with_context(|ctx| {
            ctx.on_parse
                .get_mut(&self.name)
                .map(|action| action(&t, &ast))
        });
        if let Some(Err(err)) = action_res {
            return err!(err);
        }
        Ok((r, (t, ast)))
    }
}
