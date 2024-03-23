#![feature(trait_upcasting)]
#![feature(debug_closure_helpers)]

pub mod diagnostic;
pub mod parser;
pub mod source;

mod db;
pub use db::*;

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use salsa::DebugWithDb;

    use crate::{
        diagnostic::{Diagnostics, DisplayDiagnostics},
        parser::module,
        source::SourceProgram,
    };

    use super::*;

    #[derive(Default)]
    #[salsa::db(ppl_ast::Jar, Jar)]
    pub struct Database {
        storage: salsa::Storage<Self>,
    }
    impl salsa::Database for Database {}

    #[test]
    fn test_parse_module() {
        let db = &Database::default();
        let source = SourceProgram::new(db, Some("test.ppl".into()), "fn main".to_string());

        let module = module(db, source);
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

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_parse_literals() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            r###"
            none
            true
            false
            10
            0.63
            "hello\nworld!"
            "###
            .to_string(),
        );

        let module = module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            none,
            true,
            false,
            10,
            0.63,
            "hello\nworld!",
        ]
        "###);

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn type_reference() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            r###"
            Point
            "###
            .to_string(),
        );

        let module = module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            Point,
        ]
        "###);

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_function_with_annotation() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            r###"
            @builtin
            fn main
            "###
            .to_string(),
        );

        let module = module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            [
                @builtin,
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
            ],
        ]
        "###);

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type() {
        let db = &Database::default();
        let source = SourceProgram::new(db, Some("test.ppl".into()), "type Point".to_string());

        let module = module(db, source);
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

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type_with_annotation() {
        let db = &Database::default();
        let source = SourceProgram::new(
            db,
            Some("test.ppl".into()),
            r###"
            @builtin
            type Point
            "###
            .to_string(),
        );

        let module = module(db, source);
        assert_debug_snapshot!(module.statements(db).debug_all(db), @r###"
        [
            [
                @builtin,
                Type {
                    [salsa id]: 0,
                    name: Typename {
                        [salsa id]: 0,
                        text: "Point",
                    },
                },
            ],
        ]
        "###);

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
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

        let module = module(db, source);
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

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
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

        let module = module(db, source);
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

        let diagnostics = module::accumulated::<Diagnostics>(db, source);
        assert!(diagnostics.is_empty(), "{}", diagnostics.display());
    }
}
