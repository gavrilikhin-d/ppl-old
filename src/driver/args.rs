use clap::Parser;

use super::Command;

/// PPL's package manager
#[derive(Parser, Debug)]
pub struct Args {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}
