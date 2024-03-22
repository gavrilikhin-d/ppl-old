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
    use ppl_parser::{
        diagnostic::{Diagnostics, DisplayDiagnostics},
        parser::parse_module,
        source::SourceProgram,
    };
    use salsa::DebugWithDb;

    use super::*;

    #[test]
    fn test_parse_module() {
        let db = &Database::default();
        let source = SourceProgram::new(db, Some("test.ppl".into()), "fn main".to_string());

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            Function {
                [salsa id]: 0,
                name: FunctionId {
                    [salsa id]: 0,
                    text: "main",
                },
                name_parts: [
                    Text(
                        Text(
                            Id {
                                value: 1,
                            },
                        ),
                    ),
                ],
            },
        ]
        "###);

        let diagnostics = parse_module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type() {
        let db = &Database::default();
        let source = SourceProgram::new(db, Some("test.ppl".into()), "type Point".to_string());

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            Type {
                [salsa id]: 0,
                name: Typename {
                    [salsa id]: 0,
                    text: "Point",
                },
            },
        ]
        "###);

        let diagnostics = parse_module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multipart_function() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            "fn say hello world".to_string(),
        );

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            Function {
                [salsa id]: 0,
                name: FunctionId {
                    [salsa id]: 0,
                    text: "say hello world",
                },
                name_parts: [
                    Text(
                        Text(
                            Id {
                                value: 1,
                            },
                        ),
                    ),
                    Text(
                        Text(
                            Id {
                                value: 2,
                            },
                        ),
                    ),
                    Text(
                        Text(
                            Id {
                                value: 3,
                            },
                        ),
                    ),
                ],
            },
        ]
        "###);

        let diagnostics = parse_module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty(), "{}", diagnostics.display());
    }

    #[test]
    fn test_function_with_parameter() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            "fn distance from <a: Point> to <b: Point>".to_string(),
        );

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            Function {
                [salsa id]: 0,
                name: FunctionId {
                    [salsa id]: 0,
                    text: "distance from <:Point> to <:Point>",
                },
                name_parts: [
                    Text(
                        Text(
                            Id {
                                value: 1,
                            },
                        ),
                    ),
                    Text(
                        Text(
                            Id {
                                value: 2,
                            },
                        ),
                    ),
                    Parameter(
                        Parameter {
                            name: Identifier(
                                Id {
                                    value: 1,
                                },
                            ),
                            ty: Typename(
                                Id {
                                    value: 1,
                                },
                            ),
                        },
                    ),
                    Text(
                        Text(
                            Id {
                                value: 3,
                            },
                        ),
                    ),
                    Parameter(
                        Parameter {
                            name: Identifier(
                                Id {
                                    value: 2,
                                },
                            ),
                            ty: Typename(
                                Id {
                                    value: 1,
                                },
                            ),
                        },
                    ),
                ],
            },
        ]
        "###);

        let diagnostics = parse_module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty(), "{}", diagnostics.display());
    }
}
