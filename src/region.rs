use crate::Node;

pub struct Region {
    name: String,
    /// Work tick duration
    tick: std::time::Duration,
    /// Processing nodes in this region
    nodes: Vec<Box<dyn Node>>,
}

impl Region {
    /// Construct new region.
    pub fn new(name: impl Into<String>, tick: std::time::Duration) -> Self {
        Self {
            name: name.into(),
            tick,
            nodes: Vec::new(),
        }
    }

    /// Add a node to this region
    pub fn add_node<T: Node + 'static>(&mut self, node: T) {
        self.nodes.push(Box::new(node));
    }

    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    pub(crate) fn tick(&self) -> std::time::Duration {
        self.tick
    }

    pub(crate) fn nodes_mut(&mut self) -> &mut Vec<Box<dyn Node>> {
        &mut self.nodes
    }
}
