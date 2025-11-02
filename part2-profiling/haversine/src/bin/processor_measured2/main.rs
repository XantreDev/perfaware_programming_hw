use std::{fs::File, io::Read, process::exit};

use haversine_generator::{
    json_utils,
    simple_profiler::{finish_end_print_root_profile, profile_with_label, start_profile},
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

fn main() {
    use std::env;

    let mut args = env::args();
    if args.len() < 2 {
        println!("possible args [test_data.json] [answers.fp64]?");
        exit(1);
    }

    let test_data_path = args.nth(1).expect("first argument must exist");
    let verify_file_path = args.nth(0);
    let mut json = String::new();

    start_profile();

    profile_with_label("json_io", || {
        File::open(test_data_path)
            .unwrap()
            .read_to_string(&mut json)
            .unwrap();
    });

    let json_data = profile_with_label("json_parse", || json_utils::prepare_data(json));

    let (pairs_amount, distances_sum) = profile_with_label("process", || {
        let pairs_amount = json_data.pairs.len();
        let distances_sum = process_haversine(json_data);
        (pairs_amount, distances_sum)
    });

    profile_with_label("aftermath", || {
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
    });

    finish_end_print_root_profile().unwrap();
}
