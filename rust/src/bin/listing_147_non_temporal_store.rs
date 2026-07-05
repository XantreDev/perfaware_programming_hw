use asm::non_temporal_store;
use haversine_generator::{
    core_affinity, rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

fn main() {
    core_affinity::set_single_core();

    const MAX_REP: usize = 128;
    let len: usize = 16384;
    let src = RawAlloc::new(len).as_u8_mut_ptr();
    let dst = RawAlloc::new(len * MAX_REP).as_u8_mut_ptr();
    let mut rep_tester = RepTester::new().unwrap();

    for i in 1..MAX_REP.isqrt() {
        let reps = i * i;

        let name = format!("temporal {}", i);
        rep_run!(
            rep_tester,
            name = &name,
            len = len * reps,
            block = {
                unsafe {
                    non_temporal_store::baseline_fill(len as u64, src, reps as u64, dst);
                }
            }
        );

        let name = format!("non-temporal {}", i);

        rep_run!(
            rep_tester,
            name = &name,
            len = len * reps,
            block = {
                unsafe {
                    non_temporal_store::non_temporal_fill(len as u64, src, reps as u64, dst);
                };
            }
        );
    }
}
