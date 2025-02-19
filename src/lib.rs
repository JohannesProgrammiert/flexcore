use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

pub mod region;
pub use region::Region;

pub mod node;
pub use node::Node;

pub mod ports;
pub use ports::{Input, Output};
use region::RegionBuilder;

#[derive(Default)]
/// The infrastructure manages regions and threads.
pub struct InfrastructureBuilder {
    /// Region settings
    regions: Vec<Region>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FlexcoreError {
    /// Infrastructure has no regions.
    NoRegions,
    /// Region has no nodes assigned.
    NoNodes,
}

impl InfrastructureBuilder {
    /// Add a new region/thread to the infrastructure.
    ///
    /// Call `RegionBuilder::build` to finish building the region and getting back the infrastructure handle.
    pub fn with_region(self, name: impl Into<String>, tick: std::time::Duration) -> RegionBuilder {
        RegionBuilder {
            name: name.into(),
            tick,
            nodes: Vec::new(),
            infra: self,
        }
    }

    /// Run the infrastructure
    /// Returns a `RunningInfrastructure` handle that stops the entire system when going out of scope.
    ///
    /// # Note
    ///
    /// This is non-blocking.
    pub fn build(mut self) -> Result<Infrastructure, FlexcoreError> {
        let regions = std::mem::take(&mut self.regions);
        if regions.is_empty() {
            log::error!("Infrastructure doesn't have any regions. Add at least one using `Self::with_region`.");
            return Err(FlexcoreError::NoRegions)
        }
        let mut ret = Infrastructure {
            threads: Vec::new(),
            exit_signal: Arc::new(AtomicBool::new(false)),
        };
        for mut region in regions {
            let exit = ret.exit_signal.clone();
            let name = region.name().clone();
            let builder = std::thread::Builder::new().name(region.name().clone());
            let spawn_res = builder.spawn(move || loop {
                let start_time = Instant::now();
                if exit.load(Ordering::Relaxed) {
                    return;
                }
                for node in region.nodes_mut() {
                    node.tick();
                    node.process_input();
                }
                let elapsed = start_time.elapsed();
                if elapsed > region.tick() {
                    log::warn!(
                        "Timing in region {} exceeded by {} s",
                        region.name(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    std::thread::sleep(region.tick() - elapsed);
                }
            });
            match spawn_res {
                Ok(join_hdl) => ret.threads.push(join_hdl),
                Err(e) => log::error!("Could not start thread for region {}: {e}", name),
            }
        }
        Ok(ret)
    }
}

/// Regions/threads are exited once this handle goes out of scope.
///
/// The only way of creating this object shall be `Infrastructure::run`.
pub struct Infrastructure {
    /// Thread handles after thread processing has been started
    threads: Vec<JoinHandle<()>>,
    /// Shared exit signal to stop threads
    exit_signal: Arc<AtomicBool>,
}

impl Drop for Infrastructure {
    fn drop(&mut self) {
        self.exit_signal.swap(true, Ordering::Relaxed);
        let threads = std::mem::take(&mut self.threads);
        for thr in threads {
            let name: String = thr.thread().name().unwrap_or_default().into();
            if thr.join().is_err() {
                log::warn!("Cannot join thread {}", name);
            }
        }
    }
}
