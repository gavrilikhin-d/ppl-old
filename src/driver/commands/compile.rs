use std::path::PathBuf;

use clap::Parser;

use super::Execute;
use crate::hir::Module;
use crate::ir::HIRModuleLowering;
use std::error::Error;

/// Command to compile single ppl file
#[derive(Parser, Debug)]
pub struct Compile {
    /// File to compile
    #[arg(value_name = "file")]
    pub file: PathBuf,
    /// Directory where compiler output will be placed.
    #[arg(long, value_name = "dir", default_value = ".")]
    pub output_dir: PathBuf,
}

impl Execute for Compile {
    type Output = Result<(), Box<dyn Error>>;

    /// Compile single ppl file
    fn execute(&self) -> Self::Output {
        let module = Module::from_file(&self.file)?;
        let llvm = inkwell::context::Context::create();
        let ir = module.lower_to_ir(&llvm);
        let output_file = self
            .output_dir
            .join(self.file.file_stem().unwrap())
            .with_extension("ll");
        std::fs::write(output_file, ir.to_string())?;
        Ok(())
    }
}
