use std::f64;

use haversine_generator::math;

type TrigFn = fn(value: f64) -> f64;
struct TrigEntry {
    name: &'static str,
    func: TrigFn,
}
impl TrigEntry {
    const fn new(name: &'static str, func: TrigFn) -> Self {
        Self { name, func }
    }
}
struct Range {
    start: f64,
    end: f64,
}
impl Range {
    const fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }
}
struct ComparatorTest {
    name: &'static str,
    default: TrigFn,
    other: &'static [TrigEntry],
    refs: &'static [math::references::RefPoint],
    range: Range,
}
type ComparatorTests<const N: usize> = [ComparatorTest; N];

const SIN_TEST: ComparatorTest = ComparatorTest {
    name: "sin",
    default: f64::sin,
    other: &[TrigEntry::new("same_sin", f64::sin)],
    refs: &math::references::SIN_POINTS,
    range: Range::new(0.0, f64::consts::FRAC_PI_2),
};
const COS_TEST: ComparatorTest = ComparatorTest {
    name: "cos",
    default: f64::cos,
    other: &[],
    refs: &math::references::COS_POINTS,
    range: Range::new(0.0, f64::consts::FRAC_PI_2),
};
const ASIN_TEST: ComparatorTest = ComparatorTest {
    name: "asin",
    default: f64::asin,
    other: &[],
    refs: &math::references::ASIN_POINTS,
    range: Range::new(0.0, 1.0),
};

const TESTS: ComparatorTests<3> = [SIN_TEST, COS_TEST, ASIN_TEST];

fn delta(base: TrigFn, other: TrigFn, value: f64) -> f64 {
    (base(value) - other(value)).abs()
}

fn main() {
    let mut is_first = true;
    for item in TESTS {
        if is_first {
            is_first = false;
        } else {
            print!("\n");
        }

        const COLOR_TO_GREEN: &'static str = "\x1b[38;5;118m";
        const RESET_STYLES: &'static str = "\x1b[0m";

        println!("{}{}(x){}", COLOR_TO_GREEN, item.name, RESET_STYLES);

        let default = item.default;
        {
            let mut max_delta: f64 = 0.0;
            for rref in item.refs {
                let res = default(rref.0);
                let delta = (rref.1 - res).abs();
                max_delta = max_delta.max(delta);
            }

            println!("default_max_delta={:.16}", max_delta);
        }

        for function_info in item.other {
            let mut max_delta: f64 = 0.0;
            max_delta = max_delta.max(delta(default, function_info.func, item.range.start));
            max_delta = max_delta.max(delta(default, function_info.func, item.range.end));

            let range_size = item.range.end - item.range.start;
            const POINT_COUNT: usize = 50_000;
            const K: f64 = 1.0 / POINT_COUNT as f64;

            for mult in 0..POINT_COUNT {
                let lerp_k = K * mult as f64;
                let x = item.range.start + (range_size * lerp_k);

                max_delta = max_delta.max(delta(default, function_info.func, x));
            }

            println!("max_delta({})={:.16}", function_info.name, max_delta);
        }
    }
}
