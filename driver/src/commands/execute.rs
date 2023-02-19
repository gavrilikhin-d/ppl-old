use enum_dispatch::enum_dispatch;

/// Trait for executing commands
#[enum_dispatch]
pub trait Execute {
    /// Execute the command
    fn execute(&self);
}
