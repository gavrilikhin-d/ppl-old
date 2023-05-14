use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    parsers::{ParseResult, Parser},
    Context, Pattern,
};

/// Adds name to the ast of pattern
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, From)]
pub struct Named {
    /// Name to add to the ast
    pub name: String,
    /// Pattern itself
    pub pattern: Box<Pattern>,
}

impl Parser for Named {
    fn parse_at<'s>(&self, source: &'s str, at: usize, context: &mut Context) -> ParseResult<'s> {
        let mut result = self.pattern.parse_at(source, at, context);
        result.ast = json!({&self.name: result.ast});
        result
    }
}

#[test]
fn test_named() {
    use crate::parsers::ParseResult;
    use crate::Context;
    use pretty_assertions::assert_eq;

    let mut context = Context::default();
    let pattern = Named {
        name: "name".to_string(),
        pattern: Box::new("/[A-z][a-z]*/".into()),
    };
    assert_eq!(
        pattern.parse("John", &mut context),
        ParseResult {
            delta: 4,
            tree: "John".into(),
            ast: json!({"name": "John"}),
        }
    );
}
