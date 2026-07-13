use asm::non_temporal_store;
use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester, write::RawAlloc};

fn main() {
    core_affinity::set_single_core().unwrap();

    const MIN_REP: usize = 16;
    const MAX_REP: usize = 64;
    const LEN: usize = 128 * 1024;
    let src_buf = RawAlloc::new(LEN);
    let src = src_buf.as_u8_mut_ptr();
    let dst_buf = RawAlloc::new(LEN * MAX_REP);
    let dst = dst_buf.as_u8_mut_ptr();
    let mut rep_tester = RepTester::new().unwrap();

    for i in MIN_REP.ilog2()..=MAX_REP.ilog2() {
        let reps = 2_usize.pow(i);

        let name = format!("temporal 1x{}", reps);
        rep_run!(
            rep_tester,
            name = &name,
            len = LEN * reps,
            block = {
                unsafe {
                    non_temporal_store::baseline_fill(LEN as u64, src, reps as u64, dst);
                }
            }
        );

        let name = format!("non-temporal 1x{}", reps);

        rep_run!(
            rep_tester,
            name = &name,
            len = LEN * reps,
            block = {
                unsafe {
                    non_temporal_store::non_temporal_fill(LEN as u64, src, reps as u64, dst);
                };
            }
        );
    }
}
