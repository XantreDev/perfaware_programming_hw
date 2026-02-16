use std::time::Duration;

use haversine_generator::time::TimeMeasurer;

fn main() {
    let mut measurer = TimeMeasurer::init().unwrap();

    println!("clocks={}", measurer.clocks_now());
    let frequency = measurer.detect_clock_frequency(Duration::from_millis(50));
    println!("frequency={}", frequency);
}
