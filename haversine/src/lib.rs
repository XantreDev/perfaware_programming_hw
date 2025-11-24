use std::usize;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub type PointPair = (Point, Point);

pub mod json_parser;
pub mod json_utils;
pub mod labels;
pub mod rep_tester;
pub mod simple_profiler;
pub mod time;
pub mod write;

trait Squarable {
    fn square(self) -> Self;
}
impl Squarable for f64 {
    fn square(self) -> Self {
        self * self
    }
}

pub fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, sphere_radius: f64) -> f64 {
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

pub fn pretty_print_u64(value: u64) -> String {
    pretty_print_with_options(value as f64, 0)
}
pub fn pretty_print_with_options(value: f64, after_dot: usize) -> String {
    let formatted = value.to_string();
    let mut str = String::with_capacity(formatted.len() + formatted.len() / 3);
    let point_idx = formatted.find('.').unwrap_or(formatted.len());

    let start_idx = if value >= 0.0 { 0 } else { 1 };
    let sep_idx = (point_idx - start_idx) % 3;

    // println!("{} {}", sep_idx, start_idx);
    // 1000 (4) -> 1_000
    for (idx, char) in formatted.chars().enumerate() {
        if point_idx != formatted.len() && idx.saturating_sub(point_idx) >= after_dot + 1 {
            break;
        }
        if idx < start_idx || idx == point_idx {
            str.push(char);
        } else if idx < point_idx && idx >= start_idx {
            if (idx - start_idx) % 3 == sep_idx && idx != start_idx {
                str.push('_');
            }

            str.push(char);
        } else {
            if (idx - point_idx) % 3 == 1 && idx > point_idx + 1 {
                str.push('_');
            }
            str.push(char);
        }
    }

    str
}
fn pretty_print(value: f64) -> String {
    pretty_print_with_options(value, usize::MAX - 1)
}

#[test]
fn pretty_print_test() {
    assert_eq!(pretty_print(1_000.0), "1_000");
    assert_eq!(pretty_print(1_000.1), "1_000.1");
    assert_eq!(pretty_print(-1_000.0), "-1_000");
    assert_eq!(pretty_print(100.0), "100");
    assert_eq!(pretty_print(-100.0), "-100");
    assert_eq!(pretty_print(100.5234), "100.523_4");
    assert_eq!(pretty_print(123_001_100.5234), "123_001_100.523_4");
    assert_eq!(pretty_print(1_001_100.523014), "1_001_100.523_014");
}
