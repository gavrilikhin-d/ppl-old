/// Helper macro to check that compilation happened without errors or with specified error
#[macro_export]
macro_rules! e2e {
    ($name: ident) => {
        #[test]
        fn $name() {
            use std::path::Path;

            use insta::assert_snapshot;
            use tempdir::TempDir;

            use miette::miette;

            // Compile-time check that file exists
            include_bytes!(concat!(stringify!($name), "/src/main.ppl"));

            let temp_dir = TempDir::new("ppl").unwrap();
            let file = file!();
            let name = stringify!($name);
            let dir = Path::new(file).parent().unwrap().join(name);

            let res = $crate::e2e::internal::compile(&temp_dir, &dir);
            if let Err(err) = res {
                assert_snapshot!(concat!(stringify!($name), ".error"), err);
                return;
            }

            let hir = $crate::e2e::internal::hir(&temp_dir, name, &dir);
            assert_snapshot!(concat!(stringify!($name), ".hir"), hir);

            let ir = $crate::e2e::internal::ir(&temp_dir, name, &dir);
            assert_snapshot!(concat!(stringify!($name), ".ir"), ir);

            let (run_log, status) = $crate::e2e::internal::run(&temp_dir, name, &dir);
            assert_snapshot!(concat!(stringify!($name), ".run"), run_log);
            status.exit_ok().map_err(|e| miette!("{e}")).unwrap();
        }
    };
}

#[macro_export]
macro_rules! e2es {
    ($($name: ident),+) => {
        $(
            $crate::e2e!($name);
        )+
    };
}

#[cfg(test)]
pub mod internal {
    use std::{path::Path, process::ExitStatus};

    use tempdir::TempDir;

    use crate::driver::commands::compile::OutputType;

    use miette::miette;

    pub fn compile(temp_dir: &TempDir, dir: &Path) -> Result<(), String> {
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
        let output = std::process::Command::new(ppl)
            .args(&["build"])
            .args(&["--output-dir", temp_dir_path])
            .current_dir(dir)
            .output()
            .map_err(|e| miette!("{e}"))
            .unwrap();

        let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");
        if !stderr.is_empty() {
            return Err(stderr);
        }

        Ok(())
    }

    pub fn hir(temp_dir: &TempDir, name: &str, dir: &Path) -> String {
        let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
        std::process::Command::new(ppl)
            .args(&["build"])
            .args(&["--output-dir", temp_dir.path().to_str().unwrap()])
            .args(&["--emit", "hir"])
            .current_dir(dir)
            .status()
            .map_err(|e| miette!("{e}"))
            .unwrap();

        let hir = temp_dir.path().join(OutputType::HIR.named(name));

        std::fs::read_to_string(&hir).expect("failed to read HIR")
    }

    pub fn ir(temp_dir: &TempDir, name: &str, dir: &Path) -> String {
        let ppl = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/ppl");
        std::process::Command::new(ppl)
            .args(&["build"])
            .args(&["--output-dir", temp_dir.path().to_str().unwrap()])
            .args(&["--emit", "ir"])
            .current_dir(dir)
            .status()
            .map_err(|e| miette!("{e}"))
            .unwrap();

        let ir = temp_dir.path().join(OutputType::IR.named(name));

        std::fs::read_to_string(&ir).expect("failed to read IR")
    }

    pub fn run(temp_dir: &TempDir, name: &str, dir: &Path) -> (String, ExitStatus) {
        let exe = temp_dir.path().join(OutputType::Executable.named(name));

        let output = std::process::Command::new(exe)
            .current_dir(&dir)
            .output()
            .map_err(|e| miette!("{e}"))
            .unwrap();

        let stdout = String::from_utf8(output.stdout).expect("stdout is not utf8");
        let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");

        let run_log = format!("{stdout}{stderr}");

        (run_log, output.status)
    }
}
