pub trait TerminalDisplay {
    /// Renders the current state of the simulation to the terminal.
    /// This method is called at each step of the simulation if the terminal visualizer is enabled.
    fn display(&self, step: u64);
}
