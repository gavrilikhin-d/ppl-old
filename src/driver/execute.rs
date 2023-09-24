use crate::hir::Module;
use crate::ir::HIRModuleLowering;
use log::debug;
use miette::miette;

use super::commands::compile::OutputType;
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
        let module = Module::from_file_with_builtin(&self.file, self.no_core)?;
        let llvm = inkwell::context::Context::create();
        let ir = module.lower_to_ir(&llvm);
        debug!(target: "ir", "{}", ir.to_string());

        let output_type = self.output_type.unwrap_or_else(|| {
            if ir.get_function("main").is_some() {
                OutputType::Executable
            } else {
                OutputType::DynamicLibrary
            }
        });
        let output_file = self
            .output_dir
            .join(self.file.file_stem().unwrap())
            .with_extension(output_type.extension());
        match output_type {
            OutputType::Bytecode => {
                ir.write_bitcode_to_path(&output_file);
            }
            OutputType::IR => {
                std::fs::write(&output_file, ir.to_string())
                    .map_err(|e| miette!("{output_file:?}: {e}"))?;
            }
            _ => {
                todo!("Output type {:?} not implemented", output_type)
            }
        }
        Ok(())
    }
}
