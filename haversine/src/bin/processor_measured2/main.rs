use std::{
    fs::{self, File},
    io::Read,
    os::unix::fs::MetadataExt,
    path::Path,
    process::exit,
};

use haversine_generator::{PointPair, json_utils, labels::Labels, with_label, with_profiling};

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

    with_profiling! {
        Labels =>

        with_label! {
            Labels::Args =>
            let first_arg = &args.nth(1).expect("first argument must exist");
            let test_data_path = Path::new(first_arg);
            let verify_file_path = args.nth(0);
        };



        with_label! {
            Labels::PreIO =>
            let file = File::open(test_data_path)
                .unwrap();
            let meta = file.metadata().unwrap();
        }

        with_label! {
            Labels::IO where bytes=meta.size() =>
            let json = fs::read_to_string(test_data_path).unwrap();
        };

        let json_data = json_utils::prepare_data(json);

        let pairs_amount = json_data.pairs.len();
        with_label! {
            Labels::Haversine where bytes=pairs_amount * size_of::<PointPair>() =>

            let distances_sum = process_haversine(json_data);
        };
        with_label! {
            Labels::AfterMath =>

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
        };
    };
}
