use haversine_generator::math;

type TrigFn = fn(value: f64) -> f64;
struct TrigEntry {
    name: &'static string,
    func: &TrigFn,
}
struct ComparatorTest {
    name: &'static string,
    default: TrigFn,
    other: &'static [TrigEntry],
    refs: &'static [math::references::RefPoint],
}
type ComparatorTests<const n: usize> = [ComparatorTest; n];

const SIN_TEST: ComparatorTest = ComparatorTest {
    name: "sin",
    default: f64::sin,
    other: &[],
    refs: &math::references::SIN_POINTS,
};

const TESTS: ComparatorTests = [SIN_TEST];

fn main() {
    for item in TESTS {
        let max_delta = 0.0;
        for rref in item.refs {
            let res = (item.default)(rref.0);
            max_delta;
        }
    }
}
