use std::fs;
use std::path::Path;

use crate::compilation::Compiler;
use crate::ir::HIRModuleLowering;
use crate::named::Named;
use log::{debug, trace};
use miette::miette;
use tempdir::TempDir;

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
        let mut compiler = if self.no_core {
            Compiler::without_builtin()
        } else {
            Compiler::new()
        }
        .at(self.file.parent().unwrap());

        let name = self.file.file_stem().map(|n| n.to_str()).flatten().unwrap();
        let module = compiler.compile(name)?;

        let output_type = self.output_type.unwrap_or(OutputType::Executable);
        let with_main = output_type == OutputType::Executable;

        let llvm = inkwell::context::Context::create();
        let ir = module.data(&compiler).to_ir(&llvm, with_main);
        debug!(target: "ir", "{}", ir.to_string());

        let filename = output_type.named(name);

        let output_file = self.output_dir.join(&filename);

        if output_type == OutputType::IR {
            fs::write(&output_file, ir.to_string()).map_err(|e| miette!("{output_file:?}: {e}"))?;
            return Ok(());
        }

        let temp_dir = TempDir::new("ppl").unwrap();

        let bitcode = temp_dir.path().join(filename).with_extension("bc");
        let path = module
            .data(&compiler)
            .source_file()
            .path()
            .to_string_lossy()
            .into_owned();
        trace!(target: "steps", "generating bitcode for {} => {}", path, bitcode.display());

        ir.write_bitcode_to_path(&bitcode);
        if output_type == OutputType::Bitcode {
            std::fs::copy(bitcode, output_file.with_extension("bc")).unwrap();
            return Ok(());
        }

        let bitcodes: Vec<_> = compiler
            .modules
            .values()
            .filter(|m| m.name() != name && m.name() != "ppl")
            .map(|m| {
                let llvm = inkwell::context::Context::create();
                let with_main = false;
                let ir = m.to_ir(&llvm, with_main);
                let filename = m.name().to_string();
                let bitcode = temp_dir.path().join(filename).with_extension("bc");
                trace!(target: "steps", "generating bitcode for {} => {}", m.source_file().path().to_string_lossy(), bitcode.display());
                ir.write_bitcode_to_path(&bitcode);
                bitcode.to_string_lossy().to_string()
            })
            .chain(std::iter::once(bitcode.to_string_lossy().to_string()))
            .collect();

        let mut clang = std::process::Command::new("clang");
        let lib_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/deps");
        let lib = lib_path.to_str().unwrap();

        match output_type {
            OutputType::IR => unreachable!("IR is already written"),
            OutputType::Bitcode => unreachable!("IR is already written"),

            OutputType::Object => clang.arg("-c"),
            OutputType::Assembler => clang.arg("-S"),
            OutputType::StaticLibrary => clang.args(&["-c", "-fPIC"]),
            OutputType::DynamicLibrary => clang.arg("-dynamiclib"),
            OutputType::Executable => &mut clang,
        }
        .args(&[
            "-L",
            lib,
            "-lruntime",
            if !self.no_core { "-lppl" } else { "" },
        ])
        .args(&bitcodes)
        .args(&["-o", output_file.to_str().unwrap()])
        .arg("-Wno-override-module")
        .arg("-g")
        .arg("-fsanitize=address")
        .status()
        .map_err(|e| miette!("{output_file:?}: {e}"))?;

        Ok(())
    }
}
