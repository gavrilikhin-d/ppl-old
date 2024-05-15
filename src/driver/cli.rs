use self::commands::{Build, New, Run};
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
    /// Create new package
    New(New),
    /// Build package
    Build(Build),
    /// Build and run package
    Run(Run),
}

pub mod commands {
    use std::path::PathBuf;

    use clap::Parser;

    use self::compile::OutputType;

    /// Command to create a new package
    #[derive(Parser, Debug)]
    pub struct New {
        /// New package name
        #[arg(value_name = "package")]
        pub package: String,
    }

    /// Command to build a package
    #[derive(Parser, Debug)]
    pub struct Build {
        /// Directory where compiler output will be placed.
        #[arg(long, value_name = "dir", default_value = "target")]
        pub output_dir: PathBuf,
        /// Output type of compilation
        #[arg(long = "emit", value_name = "output type")]
        pub output_type: Option<OutputType>,
    }

    impl Default for Build {
        fn default() -> Self {
            Self {
                output_dir: PathBuf::from("target"),
                output_type: None,
            }
        }
    }

    /// Command to build and run a package
    #[derive(Parser, Debug)]
    pub struct Run {}

    pub mod compile {
        use std::str::FromStr;

        use clap::ValueEnum;

        /// Output type of compilation
        #[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
        pub enum OutputType {
            HIR,
            IR,
            Bitcode,
            Object,
            Assembler,
            Executable,
            StaticLibrary,
            DynamicLibrary,
        }

        impl OutputType {
            /// Extension associated with this output type
            pub fn extension(&self) -> &'static str {
                match self {
                    Self::HIR => "hir",
                    Self::IR => "ll",
                    Self::Bitcode => "bc",
                    Self::Object => "o",
                    Self::Assembler => "s",
                    Self::Executable => {
                        if cfg!(target_os = "windows") {
                            "exe"
                        } else {
                            "out"
                        }
                    }
                    Self::StaticLibrary => {
                        if cfg!(target_os = "windows") {
                            "lib"
                        } else {
                            "a"
                        }
                    }
                    Self::DynamicLibrary => {
                        if cfg!(target_os = "windows") {
                            "dll"
                        } else if cfg!(target_os = "macos") {
                            "dylib"
                        } else {
                            "so"
                        }
                    }
                }
            }

            /// File prefix associated with this output file
            ///
            /// `Some("lib")` for [`DynamicLibrary`](OutputType::DynamicLibrary) and [`StaticLibrary`](OutputType::StaticLibrary);
            /// `None`, otherwise
            pub fn file_prefix(&self) -> Option<&'static str> {
                match self {
                    Self::StaticLibrary | Self::DynamicLibrary => Some("lib"),
                    _ => None,
                }
            }

            /// Get name of output file with correct prefix and extension
            pub fn named(&self, name: &str) -> String {
                format!(
                    "{prefix}{name}.{ext}",
                    prefix = self.file_prefix().unwrap_or(""),
                    ext = self.extension()
                )
            }
        }

        impl FromStr for OutputType {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "hir" | ".hir" => Ok(Self::HIR),
                    "ir" | ".ll" => Ok(Self::IR),
                    "bitcode" | ".bc" => Ok(Self::Bitcode),
                    "object" | ".o" => Ok(Self::Object),
                    "assembler" | ".s" => Ok(Self::Assembler),
                    "executable" | "exe" | "bin" | ".out" => Ok(Self::Executable),
                    "library" | "lib" | "static-library" | ".a" | ".lib" => Ok(Self::StaticLibrary),
                    "dynamic-library" | "dll" | "dylib" | ".so" => Ok(Self::DynamicLibrary),
                    _ => Err(()),
                }
            }
        }
    }
}
