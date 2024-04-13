use std::{
    fs,
    path::{Path, PathBuf},
};

use cmd_lib::{run_cmd, run_fun};
use log::{debug, trace};
use miette::{bail, miette};
use tempdir::TempDir;

use crate::{
    compilation::{Compiler, Package},
    driver::commands::{compile::OutputType, Build},
    ir::HIRModuleLowering,
    named::Named,
};

use super::Execute;

impl Execute for Build {
    type Output = miette::Result<()>;

    fn execute(&self) -> Self::Output {
        let cwd = std::env::current_dir().map_err(|e| miette!("{e}"))?;
        let package = cwd.file_name().unwrap().to_str().unwrap();
        let output_dir = self.output_dir.clone();

        run_cmd!(
            mkdir -p $output_dir
        )
        .map_err(|e| miette!("{e}"))?;

        let output_type = if cwd.join("src/main.ppl").exists() {
            OutputType::Executable
        } else if cwd.join("src/lib.ppl").exists() {
            OutputType::DynamicLibrary
        } else {
            bail!("No src/main.ppl or src/lib.ppl found at {}", cwd.display());
        };

        let mut compiler = if package == "ppl" {
            Compiler::without_builtin()
        } else {
            Compiler::new()
        };
        let compiler = &mut compiler;

        let package = compiler.compile_package(package)?;

        let output_type = self.output_type.unwrap_or(output_type);
        let dependencies_dir = output_dir.join("deps");
        run_cmd!(
            mkdir -p $dependencies_dir
        )
        .map_err(|e| miette!("{e}"))?;

        package.emit(compiler, output_dir, output_type, dependencies_dir)?;

        Ok(())
    }
}

trait Emit {
    fn emit(
        &self,
        compiler: &mut Compiler,
        output_dir: PathBuf,
        output_type: OutputType,
        dependencies_dir: PathBuf,
    ) -> miette::Result<PathBuf>;
}

impl Emit for Package {
    fn emit(
        &self,
        compiler: &mut Compiler,
        output_dir: PathBuf,
        output_type: OutputType,
        dependencies_dir: PathBuf,
    ) -> miette::Result<PathBuf> {
        let name = &self.data(compiler).name;
        let filename = output_type.named(name);
        let output_file = output_dir
            .canonicalize()
            .map_err(|e| miette!("Can't canonicalize output folder: {e}"))?
            .join(&filename);

        let dependencies = self.data(compiler).dependencies.clone();
        let dependencies: Vec<_> = dependencies
            .iter()
            .map(|package| {
                package.emit(
                    compiler,
                    dependencies_dir.clone(),
                    OutputType::DynamicLibrary,
                    dependencies_dir.clone(),
                )
            })
            .try_collect()?;

        let module = self.data(compiler).modules.first().unwrap().clone();
        if output_type == OutputType::HIR {
            let hir = module.data(compiler).to_string();
            fs::write(&output_file, hir)
                .map_err(|e| miette!("Can't write {output_file:?}: {e}"))?;
            return Ok(output_file);
        }

        let with_main = output_type == OutputType::Executable;

        let llvm = inkwell::context::Context::create();
        let ir = module.data(compiler).to_ir(&llvm, with_main, module);
        debug!(target: "ir", "{}", ir.to_string());
        if output_type == OutputType::IR {
            fs::write(&output_file, ir.to_string())
                .map_err(|e| miette!("Can't write {output_file:?}: {e}"))?;
            return Ok(output_file);
        }

        let temp_dir = TempDir::new("ppl").map_err(|e| miette!("Can't create tmp folder: {e}"))?;

        let bitcode = temp_dir.path().join(filename).with_extension("bc");
        let path = module
            .data(compiler)
            .source_file()
            .path()
            .to_string_lossy()
            .into_owned();
        trace!(target: "steps", "generating bitcode for {} => {}", path, bitcode.display());

        ir.write_bitcode_to_path(&bitcode);
        if output_type == OutputType::Bitcode {
            fs::copy(&bitcode, &output_file).map_err(|e| {
                miette!(
                    "Can't copy {} to {}: {e}",
                    bitcode.display(),
                    output_file.display()
                )
            })?;
            return Ok(output_file);
        }

        let bitcodes: Vec<_> = self.data(compiler)
            .modules
            .iter()
            .filter(|m| **m != module)
            .map(|m| {
                let compilation_module = m.clone();
                let m = m.data(compiler);
                let llvm = inkwell::context::Context::create();
                let with_main = false;
                let ir = m.to_ir(&llvm, with_main, compilation_module);
                let filename = m.name().to_string();
                let bitcode = temp_dir.path().join(filename).with_extension("bc");
                trace!(target: "steps", "generating bitcode for {} => {}", m.source_file().path().to_string_lossy(), bitcode.display());
                ir.write_bitcode_to_path(&bitcode);
                bitcode.to_string_lossy().to_string()
            })
            .chain(std::iter::once(bitcode.to_string_lossy().to_string()))
            .collect();

        let mut clang = std::process::Command::new("clang-17");

        let clang_version = run_fun!("clang-17" "--version")
            .map_err(|e| miette!("Valid clang-17 installation not found in path: {e}"))?;
        debug!(target: "clang", "clang version: {}", clang_version);

        let lib_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/deps");
        let lib = lib_path.to_str().unwrap();

        trace!(target: "steps", "assembling {}", output_file.display());
        let command = match output_type {
            OutputType::HIR => unreachable!("HIR is already written"),
            OutputType::IR => unreachable!("IR is already written"),
            OutputType::Bitcode => unreachable!("IR is already written"),

            OutputType::Object => clang.arg("-c"),
            OutputType::Assembler => clang.arg("-S"),
            OutputType::StaticLibrary => clang.args(&["-c", "-fPIC"]),
            OutputType::DynamicLibrary => {
                if cfg!(target_os = "macos") {
                    clang.arg("-dynamiclib")
                } else {
                    clang.args(&["-shared", "-fPIC"])
                }
            }
            OutputType::Executable => &mut clang,
        }
        .args(&["-L", lib, "-lruntime"])
        .args(&bitcodes)
        .args(dependencies)
        .arg("-Wno-override-module")
        .arg("-g")
        .arg("-fsanitize=address")
        .args(&["-o", output_file.to_str().unwrap()]);

        trace!(target: "steps", "running {:?}", command);
        command
            .status()
            .map_err(|e| miette!("Error while assembling {}: {e}", output_file.display()))?
            .exit_ok()
            .map_err(|e| {
                miette!(
                    "Error in exit status for assembling {}: {e}",
                    output_file.display()
                )
            })?;

        Ok(output_file)
    }
}
