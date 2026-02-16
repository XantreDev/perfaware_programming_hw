use haversine_generator::json_utils;

use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    process::exit,
    time::Duration,
};

fn process_haversine(data: json_utils::JsonData) -> f64 {
    let mut distances_sum = 0.0;
    let weight = 1.0 / (data.pairs.len() as f64);

    for pair in data.pairs {
        let earth_radius = 6372.8;

        let distance = haversine_generator::reference_haversine(
            pair.0.x,
            pair.0.y,
            pair.1.x,
            pair.1.y,
            earth_radius,
        );

        distances_sum += weight * distance;
    }

    distances_sum
}

struct Timestamps {
    base: u64,
    after_startup: u64,
    after_file_read: u64,
    after_json_parse: u64,
    after_process: u64,
    after_output: u64,
}
impl Timestamps {
    fn new(base: u64) -> Timestamps {
        Timestamps {
            base: base,
            after_startup: 0,
            after_file_read: 0,
            after_json_parse: 0,
            after_process: 0,
            after_output: 0,
        }
    }
}

fn format_execution_time(timestamps: &Timestamps, clock_frequency: u64) -> String {
    let total_execution_time_clocks = timestamps.after_output - timestamps.base;
    let total_time = ((total_execution_time_clocks as f64) / (clock_frequency as f64)) * 1_000.0;

    let startup_cycles = timestamps.after_startup - timestamps.base;
    let file_read_cycles = timestamps.after_file_read - timestamps.after_startup;
    let json_parse_cycles = timestamps.after_json_parse - timestamps.after_file_read;
    let processing_cycles = timestamps.after_process - timestamps.after_json_parse;
    let misc_output_cycles = timestamps.after_output - timestamps.after_process;

    let startup_percentage = (startup_cycles * 1_000 / total_execution_time_clocks) as f64 / 1000.0;
    let file_read_cycles_percentage =
        (file_read_cycles * 10_000 / total_execution_time_clocks) as f64 / 100.0;
    let json_parse_percentage =
        (json_parse_cycles * 10_000 / total_execution_time_clocks) as f64 / 100.0;

    let processing_percentage =
        (processing_cycles * 10_000 / total_execution_time_clocks) as f64 / 100.0;

    let misc_output_percentage =
        (misc_output_cycles * 10_000 / total_execution_time_clocks) as f64 / 100.0;

    format!(
        r#"
Execution time: {:.2}ms; CPU Frequency ~{}Hz
- Startup={} ({:.2})%
- IO={} ({:.2})%
- Json parsing={} ({:.2})%
- Haversine={} ({:.2})%
- Misc output={} ({:.2})%
"#,
        total_time,
        clock_frequency, //
        // pretty_print(clock_frequency as f64), //
        startup_cycles,
        startup_percentage, //
        file_read_cycles,
        file_read_cycles_percentage, //
        json_parse_cycles,
        json_parse_percentage, //
        processing_cycles,
        processing_percentage, //
        misc_output_cycles,
        misc_output_percentage //
    )
}

fn main() {
    use haversine_generator::time::TimeMeasurer;
    use std::env;

    let time_measurer = TimeMeasurer::init().unwrap();

    let mut timestamps = Timestamps::new(time_measurer.clocks_now());

    let mut args = env::args();
    if args.len() < 2 {
        println!("possible args [test_data.json] [answers.fp64]?");
        exit(1);
    }

    let first_arg = &args.nth(1).expect("first argument must exist");

    let test_data_path = Path::new(first_arg);
    let verify_file_path = args.nth(0);

    timestamps.after_startup = time_measurer.clocks_now();

    let json = fs::read_to_string(test_data_path).unwrap();

    timestamps.after_file_read = time_measurer.clocks_now();

    let json_data = json_utils::prepare_data(json);

    timestamps.after_json_parse = time_measurer.clocks_now();
    let pairs_amount = json_data.pairs.len();
    let distances_sum = process_haversine(json_data);
    timestamps.after_process = time_measurer.clocks_now();

    println!("Pairs amount: {}", pairs_amount);
    println!("Distances sum: {}", distances_sum);

    match verify_file_path {
        Some(path) => {
            let mut buf = Vec::new();
            File::open(path).unwrap().read_to_end(&mut buf).unwrap();

            if buf.len() != 8 * (pairs_amount + 1) {
                println!("invalid verify file");
                return;
            }

            let chunk = buf.last_chunk::<8>().unwrap().to_owned();
            let reference_sum = f64::from_le_bytes(chunk);

            println!("Difference: {}", distances_sum - reference_sum);
        }
        _ => {}
    }
    timestamps.after_output = time_measurer.clocks_now();

    let clock_frequency = time_measurer.detect_clock_frequency(Duration::from_millis(50));

    println!("{}", format_execution_time(&timestamps, clock_frequency));
}
