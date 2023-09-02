/// Trait for executing commands
pub trait Execute {
    /// The output of the command execution
    type Output = ();

    /// Execute the command
    fn execute(&self) -> Self::Output;
}
