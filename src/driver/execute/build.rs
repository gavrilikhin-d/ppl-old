use cmd_lib::run_cmd;
use miette::{bail, miette};

use crate::driver::commands::{compile::OutputType, Build, Compile};

use super::Execute;

impl Execute for Build {
    type Output = miette::Result<()>;

    fn execute(&self) -> Self::Output {
        let cwd = std::env::current_dir().map_err(|e| miette!("{e}"))?;
        let package = cwd.file_name().unwrap().to_str().unwrap();
        let output_dir = cwd.join("target");
        let no_core = package == "ppl";

        run_cmd!(
            mkdir -p $output_dir
        )
        .map_err(|e| miette!("{e}"))?;

        let main = cwd.join("src/main.ppl");
        if main.exists() {
            use OutputType::*;
            return Compile {
                file: main,
                output_dir,
                output_type: Some(Executable),
                no_core,
            }
            .execute();
        }

        let lib = cwd.join("src/lib.ppl");
        if lib.exists() {
            use OutputType::*;
            return Compile {
                file: lib,
                output_dir,
                output_type: Some(DynamicLibrary),
                no_core,
            }
            .execute();
        }

        bail!("No src/main.ppl or src/lib.ppl found at {}", cwd.display());
    }
}
