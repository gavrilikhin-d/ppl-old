use clap::Parser;

use crate::Command;

/// PPL's package manager
#[derive(Parser, Debug)]
pub struct Args {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}
