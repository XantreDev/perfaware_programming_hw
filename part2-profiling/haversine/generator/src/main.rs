use rand::Rng;

trait Squarable {
    fn square(self) -> Self;
}
impl Squarable for f64 {
    fn square(self) -> Self {
        self * self
    }
}

fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, sphere_radius: f64) -> f64 {
    let lon1 = x0;
    let lon2 = x1;
    let lat1 = y0;
    let lat2 = y1;

    let d_lat = lat2 - lat1;
    let d_lon = lon2 - lon1;

    let a = (d_lat.to_radians() / 2.0).sin().square()
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (d_lon.to_radians() / 2.0).sin().square();

    let c = 2.0 * a.sqrt().asin();

    let result = c * sphere_radius;
    return result;
}

struct Point {
    x: f64,
    y: f64,
}
type PointPair = (Point, Point);

fn generate_point<T: Rng>(rng: &mut T) -> Point {
    let x = {
        let x_clusters_amount = 8;
        let x_range = 360;
        let x_cluser_size = x_range / x_clusters_amount;

        let x_basis =
            x_cluser_size * (rng.random_range(0..x_clusters_amount) - x_clusters_amount / 2);
        (x_basis as f64) + rng.random_range(0.0..(x_cluser_size as f64))
    };
    let y = {
        let y_clusters_amount = 6;
        let y_range = 180;

        let y_cluster_size = y_range / y_clusters_amount;

        let y_basis =
            y_cluster_size * (rng.random_range(0..y_clusters_amount) - y_clusters_amount / 2);

        (y_basis as f64) + rng.random_range(0.0..(y_cluster_size as f64))
    };

    Point { x: x, y: y }
}

struct HaversineData {
    distances: Vec<f64>,
    pairs: Vec<PointPair>,
    distances_sum: f64,
}

// probably it will be fine to implement average like that https://stackoverflow.com/a/62939983/21157467
//
fn generate_haversine(seed: u64, pairs_amount: u32) -> HaversineData {
    let amount = pairs_amount as usize;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro128Plus;

    let mut pairs: Vec<PointPair> = Vec::with_capacity(amount);
    let mut distances: Vec<f64> = Vec::with_capacity(amount);
    let mut rng = Xoshiro128Plus::seed_from_u64(seed);

    let mut distances_sum = 0.0;
    for _ in 0..amount {
        let (a, b) = (generate_point(&mut rng), generate_point(&mut rng));
        let earth_radius = 6.378;

        let distance = reference_haversine(a.x, a.y, b.x, b.y, earth_radius);

        pairs.push((a, b));
        distances.push(distance);
        distances_sum += distance;
    }

    HaversineData {
        distances: distances,
        pairs: pairs,
        distances_sum: distances_sum,
    }
}

fn save_data(data: HaversineData, out_file_name: &str) {
    use std::fs::File;

    let mut out_file_distances =
        File::create(format!("{}.f64", out_file_name)).expect("can open file");

    for distance in data.distances {
        let binary = distance.to_bits().to_le_bytes();

        out_file_distances.write(&binary).unwrap();
    }
    out_file_distances
        .write(&data.distances_sum.to_bits().to_le_bytes())
        .unwrap();
    drop(out_file_distances);

    let mut json_file = File::create(format!("{}.json", out_file_name)).expect("can open file");
    writeln!(json_file, "{{\"pairs\": [").unwrap();

    let last_idx = data.pairs.len();
    for (idx, pair) in data.pairs.iter().enumerate() {
        write!(
            json_file,
            "  {{\"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {}}}",
            pair.0.x, pair.0.y, pair.1.x, pair.1.y
        )
        .unwrap();
        let is_last = last_idx == idx;
        if !is_last {
            writeln!(json_file, ",").unwrap();
        } else {
            writeln!(json_file, "").unwrap();
        }
    }

    writeln!(json_file, "]}}").unwrap();
}

fn main() {
    use std::env;
    if env::args().len() <= 1 {
        println!("arguments: [seed:u64] [number_of_pairs:u32]");
        return;
    }

    let seed: u64 = env::args()
        .nth(1)
        .expect("seed must be provided")
        .parse()
        .expect("must be u64");
    let number_of_pairs: u32 = env::args()
        .nth(2)
        .expect("number_of_pairs must be provided")
        .parse()
        .expect("number_of_pairs must be u32");

    use std::time::Instant;
    let start = Instant::now();

    let data = generate_haversine(seed, number_of_pairs);
    let to_generate = start.elapsed();
    let distances_sum = data.distances_sum.to_owned();
    save_data(data, &"out");
    let total = start.elapsed();
    let to_write = total - to_generate;

    println!("Seed: {}", seed);
    println!("Pairs amount: {}", number_of_pairs);
    println!("Sum of distances: {}", distances_sum);
    println!(
        "Performance: {} = {}(gen) + {}(write)",
        total.as_millis(),
        to_generate.as_millis(),
        to_write.as_millis()
    );
}
