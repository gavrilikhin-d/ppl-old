/// Helper macro to check that compilation happened without errors or with specified error
#[macro_export]
macro_rules! test_compilation_result {
    ($name: ident) => {
        #[test]
        fn $name() {
            use pretty_assertions::assert_str_eq;
            use std::path::Path;
            use tempdir::TempDir;

            let temp_dir = TempDir::new("ppl").unwrap();
            let temp_dir_path = temp_dir.path().to_str().unwrap();

            let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
            let dir = Path::new(file!())
                .parent()
                .unwrap()
                .join("tests")
                .join(stringify!($name));
            let file = concat!(stringify!($name), ".ppl");
            let output = std::process::Command::new(ppl)
                .args(&["compile", file])
                .args(&["--output-dir", temp_dir_path])
                .current_dir(dir)
                .output()
                .expect("failed to run command");

            let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");
            let expected_stderr = include_str!(concat!("tests/", stringify!($name), "/stderr.log"));
            assert_str_eq!(stderr, expected_stderr);
        }
    };
}
