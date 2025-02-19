use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;

pub mod region;
pub use region::Region;

pub mod node;
pub use node::Node;

pub mod ports;
pub use ports::{Input, Output};

#[derive(Default)]
/// The infrastructure manages regions and threads.
pub struct Infrastructure {
    /// Region settings
    regions: Vec<Region>,
}

impl Infrastructure {
    /// Add a new region/thread to the infrastructure.
    pub fn add_region(&mut self, region: Region) {
        self.regions.push(region);
    }

    /// Run the infrastructure
    /// Returns a `RunningInfrastructure` handle that stops the entire system when going out of scope.
    pub fn run(mut self) -> RunningInfrastructure {
        let regions = std::mem::take(&mut self.regions);
        let mut ret = RunningInfrastructure {
            threads: Vec::new(),
            exit_signal: Arc::new(AtomicBool::new(false)),
        };
        for mut region in regions {
            let exit = ret.exit_signal.clone();
            let name = region.name().clone();
            let builder = std::thread::Builder::new().name(region.name().clone());
            let spawn_res = builder.spawn(move || loop {
                if exit.load(Ordering::Relaxed) {
                    return;
                }
                for node in region.nodes_mut() {
                    node.tick();
                    node.process_input();
                }
                std::thread::sleep(region.tick());
            });
            match spawn_res {
                Ok(join_hdl) => ret.threads.push(join_hdl),
                Err(e) => log::error!("Could not start thread for region {}: {e}", name),
            }
        }
        ret
    }
}

/// Regions/threads are exited once this handle goes out of scope.
///
/// The only way of creating this object shall be `Infrastructure::run`.
pub struct RunningInfrastructure {
    /// Thread handles after thread processing has been started
    threads: Vec<JoinHandle<()>>,
    /// Shared exit signal to stop threads
    exit_signal: Arc<AtomicBool>,
}

impl Drop for RunningInfrastructure {
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
