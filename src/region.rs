use crate::{FlexcoreError, InfrastructureBuilder, Node};

pub struct RegionBuilder {
    pub(crate) name: String,
    pub(crate) tick: std::time::Duration,
    pub(crate) nodes: Vec<Box<dyn Node>>,
    pub(crate) infra: InfrastructureBuilder
}

impl RegionBuilder {
    /// Add a node to this region
    pub fn with_node<T: Node + 'static>(mut self, node: T) -> Self {
        self.nodes.push(Box::new(node));
        self
    }

    pub fn build(mut self) -> Result<InfrastructureBuilder, FlexcoreError> {
        if self.nodes.is_empty() {
            log::error!("Region {} has no nodes assigned. Please assign at least one node using `Self::with_node`", self.name);
            return Err(FlexcoreError::NoNodes)
        }
        let region = Region {
            name: self.name,
            tick: self.tick,
            nodes: self.nodes
        };
        self.infra.regions.push(region);
        Ok(self.infra)
    }
}

pub struct Region {
    name: String,
    /// Work tick duration
    tick: std::time::Duration,
    /// Processing nodes in this region
    nodes: Vec<Box<dyn Node>>,
}

impl Region {
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
