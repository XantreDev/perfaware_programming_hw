use std::time::Duration;

use haversine_generator::time::{clocks_now, detect_clock_frequency};

fn main() {
    let clocks = clocks_now();
    println!("clocks={}", clocks);
    let frequency = detect_clock_frequency(Duration::from_millis(50));
    println!("frequency={}", frequency);
}
