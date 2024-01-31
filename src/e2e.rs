/// Helper macro to check that compilation happened without errors or with specified error
#[macro_export]
macro_rules! e2e {
    ($name: ident) => {
        #[test]
        fn $name() {
            $crate::e2e::internal::e2e(file!(), stringify!($name))
        }
    };
}

#[cfg(test)]
pub mod internal {
    use std::fs;

    use crate::driver::commands::compile::OutputType;

    pub fn e2e(file: &str, name: &str) {
        use pretty_assertions::assert_str_eq;
        use std::path::Path;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("ppl").unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
        let dir = Path::new(file).parent().unwrap().join(name);
        let file = format!("{name}.ppl");
        let output = std::process::Command::new(ppl)
            .args(&["compile", &file])
            .args(&["--output-dir", temp_dir_path])
            .current_dir(&dir)
            .output()
            .expect("failed to run command");

        let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");
        let expected_stderr = fs::read_to_string(format!("{dir}/stderr.log", dir = dir.display()))
            .unwrap_or_default();
        assert_str_eq!(stderr, expected_stderr, "compiler output should match");

        if !expected_stderr.is_empty() {
            return;
        }

        let exe = temp_dir.path().join(OutputType::Executable.named(name));
        let output = std::process::Command::new(exe)
            .current_dir(&dir)
            .output()
            .expect("failed to run executable");

        let run_log = String::from_utf8(output.stdout).expect("stdout is not utf8");
        let expected_run_log =
            fs::read_to_string(format!("{dir}/run.log", dir = dir.display())).unwrap_or_default();
        assert_str_eq!(run_log, expected_run_log, "executable output should match");
    }
}
