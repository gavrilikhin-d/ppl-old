#[derive(Default)]
#[salsa::db(ppl_ast::Jar, ppl_parser::Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}
impl salsa::Database for Database {}
impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Database {
            storage: self.storage.snapshot(),
        })
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use ppl_parser::{parser::parse_module, source::SourceProgram};
    use salsa::DebugWithDb;

    use super::*;

    #[test]
    fn test_parse_module() {
        let db = &Database::default();
        let source = SourceProgram::new(db, "fn main".to_string());

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug(db), @r###"
        [
            Function {
                [salsa id]: 0,
                name: FunctionId {
                    [salsa id]: 0,
                    text: "main",
                },
            },
        ]
        "###);
    }
}
