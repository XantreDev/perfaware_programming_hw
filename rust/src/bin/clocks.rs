use std::time::Duration;

use haversine_generator::time::TimeMeasurer;

fn main() {
    let measurer = TimeMeasurer::init().unwrap();
    let frequency = measurer.detect_clock_frequency(Duration::from_millis(100));

    println!("RDTSC frequency is {}", frequency);
}
