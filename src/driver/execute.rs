use crate::hir::Module;
use crate::ir::HIRModuleLowering;
use miette::miette;

use super::commands::Compile;
use super::Command;

/// Trait for executing commands
pub trait Execute {
    /// The output of the command execution
    type Output = ();

    /// Execute the command
    fn execute(&self) -> Self::Output;
}

impl Execute for Command {
    type Output = miette::Result<()>;

    fn execute(&self) -> Self::Output {
        match self {
            Command::Compile(compile) => compile.execute(),
        }
    }
}

impl Execute for Compile {
    type Output = miette::Result<()>;

    /// Compile single ppl file
    fn execute(&self) -> Self::Output {
        let module = Module::from_file(&self.file)?;
        let llvm = inkwell::context::Context::create();
        let ir = module.lower_to_ir(&llvm);
        let output_file = self
            .output_dir
            .join(self.file.file_stem().unwrap())
            .with_extension("ll");
        std::fs::write(&output_file, ir.to_string())
            .map_err(|e| miette!("{output_file:?}: {e}"))?;
        Ok(())
    }
}
