use haversine_generator::{
    core_affinity, rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

fn main() {
    core_affinity::set_single_core();

    const BUF_BITS: usize = 30;
    const BUF_SIZE: usize = 1 << BUF_BITS;
    let memory = RawAlloc::new(BUF_SIZE);
    let buffer = memory.as_u8_slice_mut();
    const STEP_BITS: usize = 3;

    let mut rep_tester = RepTester::new().unwrap();
    for i in 10..30 {
        for step in 0..(1 << STEP_BITS) {
            let size = (1 << i) + (step << (i - STEP_BITS));
            let name: String;
            if (i - 3) >= 20 {
                name = format!("cache_{}MB", size >> 20);
            } else if (i - 3) >= 10 {
                name = format!("cache_{}kB", size >> 10);
            } else {
                name = format!("cache_{}B", size);
            }
            let len = (buffer.len() as f64 / size as f64).ceil() as u64;
            rep_run!(rep_tester, name = &name, len = len, block = { unsafe {} });
        }
    }
}
