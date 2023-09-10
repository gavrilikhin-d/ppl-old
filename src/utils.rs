/// Helper macro to check that compiler error is correct
#[macro_export]
macro_rules! test_compiler_error {
    ($name: ident) => {
        #[test]
        fn $name() {
            use pretty_assertions::assert_str_eq;
            use std::path::Path;

            let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
            let dir = Path::new(file!())
                .parent()
                .unwrap()
                .join("tests")
                .join(stringify!($name));
            let file = concat!(stringify!($name), ".ppl");
            let output = std::process::Command::new(ppl)
                .args(&["compile", file])
                .current_dir(dir)
                .output()
                .expect("failed to run command");

            let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");
            let expected_stderr = include_str!(concat!("tests/", stringify!($name), "/stderr.log"));
            assert_str_eq!(stderr, expected_stderr);
        }
    };
}
