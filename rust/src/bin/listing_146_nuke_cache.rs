use std::env::args;

use asm::nuke_cache;
use haversine_generator::{
    IntParsableStr, core_affinity, rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

fn main() {
    // const CACHE_PAGE_ENTRIES: u64 = 1 << 6;
    // const L1_SETS: u64 = 64;
    // const L1_ASSOCIATIVITY: u64 = 8;
    let size = {
        let size = args()
            .nth(1)
            .expect("must pass size")
            .parse_int::<u32>("must pass size");
        assert!(((size as u64) & (nuke_cache::L1_ITERATION_READ_BYTES as u64 - 1)) == 0);
        assert!(size > 0);

        size
    };

    core_affinity::set_single_core().unwrap();
    let buf = RawAlloc::new(nuke_cache::L1_NUKE_REQUIRED_MEMORY);
    let mut rep_tester = RepTester::new().unwrap();

    loop {
        let iterations = size as usize / nuke_cache::L1_ITERATION_READ_BYTES;
        rep_run!(
            rep_tester,
            name = "nuke l1",
            len = size,
            block = {
                unsafe {
                    nuke_cache::nuke_l1(iterations as u64, buf.as_mut_ptr() as *mut u64);
                };
            }
        );
    }
}
