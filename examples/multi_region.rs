use std::time::Duration;

use flexcore::*;

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

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let mut sensor_interface = SensorInterface::new("counter");
    let mut processing = Processing::new("processing");
    let mut business_logic = BusinessLogic::new("output");

    sensor_interface
        .out_measurements
        .connect(&mut processing.in_measurements);

    processing
        .out_velocity
        .connect(&mut business_logic.in_velocity);

    let infra = InfrastructureBuilder::default();
    let _handle = infra
        .with_region("Sensor", Duration::from_secs_f64(0.1))
        .with_node(sensor_interface)
        .build()
        .unwrap()

        .with_region("Processing", Duration::from_secs_f64(0.3))
        .with_node(processing)
        .build()
        .unwrap()

        .with_region("Final", Duration::from_secs_f64(0.3))
        .with_node(business_logic)
        .build()
        .unwrap()

        .build()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(3));
}
