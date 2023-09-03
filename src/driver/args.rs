use clap::{Parser, Subcommand};
use derive_more::From;

/// PPL's package manager
#[derive(Parser, Debug)]
pub struct Args {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The subcommands of ppl
#[derive(Subcommand, Debug, From)]
pub enum Command {
    /// Compile single ppl file
    Compile(Compile),
}

pub mod commands {
    use std::path::PathBuf;

    use clap::Parser;

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
}
use self::commands::Compile;
