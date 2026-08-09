use std::{
    fs::{self, File},
    io::Read,
    os::unix::fs::MetadataExt,
    path::Path,
    process::exit,
};

use haversine_generator::{
    PointPair, core_affinity, json_utils, labels::Labels, with_label, with_profiling,
    write::RawAlloc,
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
    core_affinity::set_single_core().unwrap();

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
            let mut file = File::open(test_data_path)
                .unwrap();
            let meta = file.metadata().unwrap();
        }

        // for i in (0..arr.len()).step_by(4096) {
        //     arr[i] = 0;
        // }
        with_label! {
            Labels::IO where bytes=meta.size() =>

            // let mut vec: Vec<u8> = Vec::with_capacity(meta.size() as usize + 64);
            // let arr = unsafe {
            //     let alignment = 63;
            //    &mut vec.spare_capacity_mut().assume_init_mut()[alignment..meta.size() as usize + alignment]
            // };
            //
            // println!("ptr={:p}, offset={}", arr.as_ptr(), arr.as_ptr() as usize & 4095);
            // file.read_exact(arr).unwrap();
            // wtf? raw mmap 2x faster
            // let buf = RawAlloc::new((meta.size() + 64) as usize);
            // let json = &mut buf.as_u8_slice_mut()[16..16 + meta.size() as usize];
            let buf = RawAlloc::new((meta.size()) as usize);
            let arr = &mut buf.as_u8_slice_mut();
            file.read_exact(arr).unwrap();
            let json = arr;
        };

        let json_data = json_utils::prepare_data_from_slice(json);

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
