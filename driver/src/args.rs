use clap::Parser;

use crate::Command;

/// PPL's package manager
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
