use haversine_generator::{core_affinity, write::RawAlloc};

fn main() {
    core_affinity::set_single_core();

    const MAX_REP: usize = 128;
    let buf1 = RawAlloc::new(16384);
    let buf2 = RawAlloc::new(16384 * MAX_REP);

    for i in 1..MAX_REP.isqrt() {
        let reps = i * i;
    }
}
