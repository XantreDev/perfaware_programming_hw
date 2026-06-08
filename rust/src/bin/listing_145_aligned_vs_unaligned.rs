use std::env::args;

use haversine_generator::{
    core_affinity, rep_run,
    rep_tester::{MeasurementKind, PerformanceMeasurement, RepTester},
    write::RawAlloc,
};

struct LoadCacheKind {
    name: &'static str,
    misalignment: u8,
}
impl LoadCacheKind {
    fn new(name: &'static str, misalignment: u8) -> Self {
        Self { name, misalignment }
    }
}

fn main() {
    let (size, is_synthetic, is_csv) = {
        let size: u32 = args()
            .nth(1)
            .expect("must pass array size")
            .replace("_", "")
            .parse()
            .expect("valid u32");
        let kind = args().nth(2).expect("must pass 'synthetic/real'");
        let output = args().nth(3).expect("must pass 'csv/print'");
        let size = size as u64;
        assert!(size & 127 == 0);

        assert!(output == "csv" || output == "print");
        assert!(kind == "synthetic" || kind == "real");

        (size, kind == "synthetic", output == "csv")
    };
    core_affinity::set_single_core().expect("must set");

    let buf_size = size as usize + 8;
    let alloc = RawAlloc::new(buf_size);
    let buf = alloc.as_u8_mut_ptr();
    let mut rep_tester = RepTester::new().unwrap();

    for i in 0..buf_size {
        unsafe {
            *buf.wrapping_add(i) = i as u8;
        };
    }

    let mut loads_measurements = [PerformanceMeasurement::default(); 5];
    let misalign_loads = [
        LoadCacheKind::new("misalign 0", 0), // baseline
        LoadCacheKind::new("misalign 1", 1),
        LoadCacheKind::new("misalign 32", 32),
        LoadCacheKind::new("misalign 42", 42),
        LoadCacheKind::new("misalign 63", 63),
    ];
    rep_tester.print = !is_csv;
    for (idx, item) in misalign_loads.iter().enumerate() {
        rep_run!(
            rep_tester,
            name = item.name,
            len = size,
            block = {
                unsafe {
                    let ptr = buf.byte_add(item.misalignment as usize) as *mut u64;
                    if is_synthetic {
                        asm::simd::read_32x3(size, ptr);
                    } else {
                        asm::non_bin_cache::test_cache_non_bin(
                            1,
                            size / asm::non_bin_cache::READ_SIZE as u64,
                            ptr,
                        );
                    }
                }
            },
        );
        let collected = rep_tester.measurement(MeasurementKind::Best);

        loads_measurements[idx] = collected;
    }
}
