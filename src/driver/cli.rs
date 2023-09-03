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

    use self::compile::OutputType;

    /// Command to compile single ppl file
    #[derive(Parser, Debug)]
    pub struct Compile {
        /// File to compile
        #[arg(value_name = "file")]
        pub file: PathBuf,
        /// Directory where compiler output will be placed.
        #[arg(long, value_name = "dir", default_value = ".")]
        pub output_dir: PathBuf,
        /// Output type of compilation
        #[arg(long = "emit", value_name = "output type", default_value = "bytecode")]
        pub output_type: Option<OutputType>,
        /// Compile without core library.
        /// Used for compiling core library itself.
        #[arg(long, default_value = "false")]
        pub no_core: bool,
    }

    pub mod compile {
        use std::str::FromStr;

        use clap::ValueEnum;

        /// Output type of compilation
        #[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
        pub enum OutputType {
            IR,
            Bytecode,
        }

        impl OutputType {
            /// Extension associated with this output type
            pub fn extension(&self) -> &'static str {
                match self {
                    Self::IR => "ll",
                    Self::Bytecode => "bc",
                }
            }
        }

        impl FromStr for OutputType {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "ir" => Ok(Self::IR),
                    "bytecode" => Ok(Self::Bytecode),
                    _ => Err(()),
                }
            }
        }

        impl Default for OutputType {
            fn default() -> Self {
                Self::Bytecode
            }
        }
    }
}
use self::commands::Compile;
