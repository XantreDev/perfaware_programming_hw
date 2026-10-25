use std::{
    env::args,
    fs::{File, read_to_string},
    io::Read,
};

use haversine_generator::{
    PointPair, json_utils::prepare_haversinve_data_from_json, reference_haversine, rep_run,
    setup_rep_test_single_core,
};

struct HaversineArgs {
    case_basename: String,
    is_debug: bool,
}

struct ReferenceData {
    distances: Vec<f64>,
    sum: f64,
}
struct ImplementationVariant {
    name: &'static str,
    process_many: fn(data: &[PointPair]) -> f64,
    process: fn(pair: &PointPair) -> f64,
}
impl ImplementationVariant {
    fn process_many(&self, data: &[PointPair]) -> f64 {
        let process_many = self.process_many;
        process_many(data)
    }

    fn process(&self, data: &PointPair) -> f64 {
        let process = self.process;
        process(data)
    }
}
const EARTH_RADIUS: f64 = 6372.8;

const REFERENCE: ImplementationVariant = ImplementationVariant {
    name: "reference",
    process_many: |data: &[PointPair]| -> f64 {
        let mut total = 0.0;
        let k = 1.0 / data.len() as f64;

        for pair in data {
            total += k * reference_haversine(pair.0.x, pair.0.y, pair.1.x, pair.1.y, EARTH_RADIUS);
        }

        total
    },
    process: |pair| -> f64 {
        reference_haversine(pair.0.x, pair.0.y, pair.1.x, pair.1.y, EARTH_RADIUS)
    },
};
const OPTIONS: &[ImplementationVariant] = &[REFERENCE];

fn main() {
    let mut tester = setup_rep_test_single_core().unwrap();
    let args = {
        let mut args: Vec<String> = args().skip(1).collect();

        let is_debug = args.contains(&String::from("--debug"));
        let path = args
            .iter()
            .nth(0)
            .unwrap_or_else(|| panic!("First argument must be case basename"));

        HaversineArgs {
            case_basename: path.to_owned(),
            is_debug: is_debug,
        }
    };

    let reference_data = {
        let mut buf = Vec::new();
        File::open(format!("{}.f64", args.case_basename))
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();

        const F64_SIZE: usize = 8;
        assert!(buf.len() % F64_SIZE == 0);
        let pairs_count = (buf.len() / F64_SIZE) - 1;

        assert!(pairs_count > 1, "must have non empty input");
        let mut reference_data = ReferenceData {
            distances: Vec::with_capacity(pairs_count),
            sum: 0.0,
        };
        for i in (0..(buf.len() - F64_SIZE)).step_by(F64_SIZE) {
            let buf: &[u8; F64_SIZE] = buf[i..].first_chunk().unwrap();

            reference_data
                .distances
                .push(f64::from_le_bytes(buf.to_owned()));
        }
        reference_data.sum = f64::from_le_bytes(
            buf[F64_SIZE * pairs_count..]
                .first_chunk()
                .unwrap()
                .to_owned(),
        );

        reference_data
    };
    let json_buf = {
        let mut buf = Vec::new();
        File::open(format!("{}.json", args.case_basename))
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        buf
    };
    let json_data = prepare_haversinve_data_from_json(&json_buf);

    let size = size_of::<PointPair>() * json_data.pairs.len();

    for item in OPTIONS {
        let mut last: Option<f64> = None;
        rep_run!(
            tester,
            name = item.name,
            len = size,
            block = {
                last = Some(item.process_many(&json_data.pairs));
            }
        );

        let last = last.unwrap_or_else(|| item.process_many(&json_data.pairs));
        const EPS: f64 = 1e-6;
        let delta = reference_data.sum - last;
        if delta.abs() < EPS {
            println!("{} delta is fine", item.name);
            continue;
        }

        println!("{} delta is not ok: (|{}| > {})", item.name, delta, EPS);
        // TODO: debug
    }
}
