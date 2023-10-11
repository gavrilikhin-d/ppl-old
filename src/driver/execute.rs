use std::fs;
use std::path::Path;

use crate::compilation::Compiler;
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
        let mut compiler = Compiler::new().at(self.file.parent().unwrap());
        println!("{}", compiler.root.to_str().unwrap());
        compiler.is_builtin = self.no_core;

        let name = self.file.file_stem().map(|n| n.to_str()).flatten().unwrap();
        let module = compiler.get_module(name)?;

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

        if output_type == OutputType::IR {
            fs::write(&output_file, ir.to_string()).map_err(|e| miette!("{output_file:?}: {e}"))?;
            return Ok(());
        }

        let bitcode = output_file.with_extension("bc");
        ir.write_bitcode_to_path(&bitcode);
        if output_type == OutputType::Bitcode {
            return Ok(());
        }
        let bitcode = bitcode.to_str().unwrap();

        let mut clang = std::process::Command::new("clang");
        let runtime_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/deps");
        let runtime = runtime_path.to_str().unwrap();

        match output_type {
            OutputType::IR => unreachable!("IR is already written"),
            OutputType::Bitcode => unreachable!("IR is already written"),

            OutputType::Object => clang.arg("-c"),
            OutputType::Assembler => clang.arg("-S"),
            OutputType::StaticLibrary => {
                clang
                    .args(&["-c", "-fPIC"])
                    .args(&["-L", runtime, "-lruntime"])
            }
            OutputType::DynamicLibrary => {
                clang
                    .args(&["-c", "-fPIC", "-shared"])
                    .args(&["-L", runtime, "-lruntime"])
            }
            OutputType::Executable => clang.args(&["-L", runtime, "-lruntime"]),
        }
        .arg(bitcode)
        .args(&["-o", output_file.to_str().unwrap()])
        .arg("-Wno-override-module")
        .status()
        .map_err(|e| miette!("{output_file:?}: {e}"))?;

        fs::remove_file(&bitcode).unwrap();

        Ok(())
    }
}
