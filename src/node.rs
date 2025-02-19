pub trait Node: Send {
    fn name(&self) -> &String;

    /// Specify special behavior that should happen on each tick, e.g. reading a device
    /// or firing on output nodes.
    ///
    /// Per default, this is noop.
    fn tick(&mut self) {}

    /// Here the use shall read all `Input` ports, process the data accordingly,
    /// and fire outputs that are related to it.
    ///
    /// # TODO
    ///
    /// Input reading should happen automatically at each tick.
    /// The user should specify what to do with the received data.
    fn process_input(&mut self);
}
