use clap::Subcommand;
use enum_dispatch::enum_dispatch;

mod execute;
pub use execute::*;

mod new;
pub use new::*;

mod init;
pub use init::*;

#[derive(Subcommand, Debug)]
#[enum_dispatch(Execute)]
pub enum Command {
    /// Create a new ppl project at <path>
    New(New),
    /// Create a new ppl package in an existing directory
    Init(Init),
}
