use insta::assert_snapshot;
use ppl_ast::display::DisplayWithDb;

use crate::{diagnostic::Diagnostics, parser::module, source::SourceProgram};

use super::*;

#[derive(Default)]
#[salsa::db(ppl_ast::Jar, Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}
impl salsa::Database for Database {}

macro_rules! tests {
    ($($name: ident),+) => {
        $(test!($name);)+
    };
}

macro_rules! test {
    ($name: ident) => {
        #[test]
        fn $name() {
            let text = include_str!(concat!(stringify!($name), ".ppl"));
            test_impl(concat!(stringify!($name), ".ppl"), text);
        }
    };
}

fn test_impl(source: &str, text: &str) {
    let db = &Database::default();
    let source = SourceProgram::new(db, Some(source.into()), text.to_string());

    let module = module(db, source);
    assert_snapshot!(module
        .statements(db)
        .iter()
        .map(|s| s.to_string_with(db))
        .collect::<Vec<_>>()
        .join("\n"));

    let diagnostics = module::accumulated::<Diagnostics>(db, source);
    assert!(diagnostics.is_empty());
}

tests!(declarations, expressions);
