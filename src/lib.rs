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

/// The infrastructure manages regions and threads.
pub struct Infrastructure {
    /// Region settings
    regions: Vec<Region>,
}

impl Infrastructure {
    /// Create new infrastructure with default values.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

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
            let join_hdl = std::thread::Builder::new()
                .name(region.name().clone())
                .spawn(move || loop {
                    if exit.load(Ordering::Relaxed) {
                        return;
                    }
                    for node in region.nodes_mut() {
                        node.tick();
                        node.process_input();
                    }
                    std::thread::sleep(region.tick());
                })
                .expect("Could not launch thread");
            ret.threads.push(join_hdl);
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
            thr.join().expect("Cannot join thread");
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Default, Debug, Clone)]
    struct SensorData {
        pub data: [f64; 3],
    }

    struct SensorInterface {
        name: String,
        counter: usize,
        out_measurements: Output<SensorData>,
    }

    impl SensorInterface {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                counter: 0,
                out_measurements: Output::default(),
            }
        }
    }

    impl Node for SensorInterface {
        fn name(&self) -> &String {
            &self.name
        }
        fn tick(&mut self) {
            println!("Sensor counter: {}", self.counter);
            let v: f64 = self.counter as f64;
            self.counter += 1;
            self.out_measurements.fire(SensorData {
                data: [v, v + 1.0, v + 2.0],
            });
        }
        fn process_input(&mut self) {}
    }
    struct Processing {
        name: String,
        in_measurements: Input<SensorData>,
        out_velocity: Output<f64>,
    }

    impl Processing {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                in_measurements: Input::default(),
                out_velocity: Output::default(),
            }
        }
    }

    impl Node for Processing {
        fn name(&self) -> &String {
            &self.name
        }
        fn process_input(&mut self) {
            // This should happen automatically.
            // Ideally, I'd want a trait
            //
            // fn process_input(&mut self, data: T);
            let data = self.in_measurements.fetch();

            for d in data {
                // only this inner part should be user-specified.
                let velocity = d.data[0] as f64 * d.data[1] as f64 * d.data[2] as f64;
                println!("{:?} -> {}", d, velocity);
                self.out_velocity.fire(velocity);
            }
        }
    }

    struct BusinessLogic {
        name: String,
        in_velocity: Input<f64>,
    }

    impl BusinessLogic {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                in_velocity: Input::default(),
            }
        }
    }

    impl Node for BusinessLogic {
        fn name(&self) -> &String {
            &self.name
        }
        fn process_input(&mut self) {
            let data = self.in_velocity.fetch();
            for d in data {
                println!("Velocity: {}", d);
            }
        }
    }

    #[test]
    fn single_region() {
        let mut sensor_interface = SensorInterface::new("counter");
        let mut processing = Processing::new("processing");
        let mut business_logic = BusinessLogic::new("output");

        sensor_interface
            .out_measurements
            .connect(&mut processing.in_measurements);

        processing
            .out_velocity
            .connect(&mut business_logic.in_velocity);

        let mut r1 = Region::new("Sensor", std::time::Duration::from_secs(1));

        r1.add_node(sensor_interface);
        r1.add_node(processing);
        r1.add_node(business_logic);

        assert_eq!(r1.nodes_mut().len(), 3);

        let mut infra = Infrastructure::new();
        infra.add_region(r1);

        assert_eq!(infra.regions.len(), 1);

        let _handle = infra.run();
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
