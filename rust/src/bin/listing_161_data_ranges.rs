use std::{env::args, f64, fs::File, io::Read};

use haversine_generator::{
    PointPair, core_affinity, json_utils::prepare_haversinve_data_from_json,
};

trait Squarable {
    fn square(self) -> Self;
}
impl Squarable for f64 {
    fn square(self) -> Self {
        self * self
    }
}

const EARTH_RADIUS: f64 = 6372.8;

#[derive(Debug)]
struct MinMax {
    min: f64,
    max: f64,
}
impl Default for MinMax {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
        }
    }
}
impl MinMax {
    fn track(&mut self, val: &f64) {
        let mm = self;
        mm.max = val.max(mm.max);
        mm.min = val.min(mm.min);
    }
}
#[derive(Debug, Default)]
struct HaversineCalls {
    sin: MinMax,
    cos: MinMax,
    asin: MinMax,
}
trait TrackedTrig {
    fn tsin(self, mm: &mut HaversineCalls) -> Self;
    fn tcos(self, mm: &mut HaversineCalls) -> Self;
    fn tasin(self, mm: &mut HaversineCalls) -> Self;
}
impl TrackedTrig for f64 {
    fn tsin(self, mm: &mut HaversineCalls) -> Self {
        mm.sin.track(&self);

        self.sin()
    }

    fn tcos(self, mm: &mut HaversineCalls) -> Self {
        mm.cos.track(&self);

        self.cos()
    }

    fn tasin(self, mm: &mut HaversineCalls) -> Self {
        mm.asin.track(&self);

        self.asin()
    }
}

fn thaversine(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    sphere_radius: f64,
    t: &mut HaversineCalls,
) -> f64 {
    let lon1 = x0;
    let lon2 = x1;
    let lat1 = y0;
    let lat2 = y1;

    let d_lat = lat2 - lat1;
    let d_lon = lon2 - lon1;

    let a = (d_lat.to_radians() / 2.0).tsin(t).square()
        + lat1.to_radians().tcos(t)
            * lat2.to_radians().tcos(t)
            * (d_lon.to_radians() / 2.0).tsin(t).square();

    let c = 2.0 * a.sqrt().tasin(t);

    let result = c * sphere_radius;
    return result;
}

fn process_many(data: &[PointPair]) -> (f64, HaversineCalls) {
    let mut total = 0.0;
    let mut haversine_calls = HaversineCalls::default();
    let k = 1.0 / data.len() as f64;

    for pair in data {
        total += k * thaversine(
            pair.0.x,
            pair.0.y,
            pair.1.x,
            pair.1.y,
            EARTH_RADIUS,
            &mut haversine_calls,
        );
    }

    (total, haversine_calls)
}

fn main() {
    core_affinity::set_single_core().unwrap();
    let case_basepath = {
        let args: Vec<String> = args().skip(1).collect();

        let path = args
            .iter()
            .nth(0)
            .unwrap_or_else(|| panic!("First argument must be case basename"));

        path.to_owned()
    };

    let json_buf = {
        let mut buf = Vec::new();
        File::open(format!("{}.json", case_basepath))
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        buf
    };
    let json_data = prepare_haversinve_data_from_json(&json_buf);

    let (_, calls) = process_many(&json_data.pairs);
    println!("{:?}", calls);
}
