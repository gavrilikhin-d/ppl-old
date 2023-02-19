/// Trait for executing commands
pub trait Execute {
    /// The return type of the command
    type ReturnType = ();

    /// Execute the command
    fn execute(&self) -> Self::ReturnType;
}
