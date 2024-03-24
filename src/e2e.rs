/// Helper macro to check that compilation happened without errors or with specified error
#[macro_export]
macro_rules! e2e {
    ($name: ident) => {
        #[test]
        fn $name() {
            use std::path::Path;

            use insta::assert_snapshot;
            use tempdir::TempDir;

            // Compile-time check that file exists
            include_bytes!(concat!(stringify!($name), "/", stringify!($name), ".ppl"));

            let temp_dir = TempDir::new("ppl").unwrap();
            let file = file!();
            let name = stringify!($name);
            let dir = Path::new(file).parent().unwrap().join(name);

            let res = $crate::e2e::internal::compile(&temp_dir, name, &dir);
            if let Err(err) = res {
                assert_snapshot!(concat!(stringify!($name), ".error"), err);
                return;
            }

            let run_log = $crate::e2e::internal::run(&temp_dir, name, &dir);
            assert_snapshot!(concat!(stringify!($name), ".run"), run_log);
        }
    };
}

#[cfg(test)]
pub mod internal {
    use std::path::Path;

    use tempdir::TempDir;

    use crate::driver::commands::compile::OutputType;

    pub fn compile(temp_dir: &TempDir, name: &str, dir: &Path) -> Result<(), String> {
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
        let file = format!("{name}.ppl");
        let output = std::process::Command::new(ppl)
            .args(&["compile", &file])
            .args(&["--output-dir", temp_dir_path])
            .current_dir(dir)
            .output()
            .expect("failed to run command");

        let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");
        if !stderr.is_empty() {
            return Err(stderr);
        }

        Ok(())
    }

    pub fn run(temp_dir: &TempDir, name: &str, dir: &Path) -> String {
        let exe = temp_dir.path().join(OutputType::Executable.named(name));

        let output = std::process::Command::new(exe)
            .current_dir(&dir)
            .output()
            .expect("failed to run executable");

        let stdout = String::from_utf8(output.stdout).expect("stdout is not utf8");
        let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");

        let run_log = format!("{stdout}{stderr}");

        output.status.exit_ok().unwrap();

        run_log
    }
}
