use asm::prefetching;
use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester, write::RawAlloc};
use rand::{Rng, SeedableRng, random};
use rand_xoshiro::Xoshiro128PlusPlus;

fn packed_access_pattern(accesses: usize, size: usize) -> Vec<u32> {
    assert!(accesses % 4 == 0);
    let mut rand1 =
        Xoshiro128PlusPlus::from_seed([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let mut rand2 =
        Xoshiro128PlusPlus::from_seed([1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let mut rand3 =
        Xoshiro128PlusPlus::from_seed([2, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let mut rand4 =
        Xoshiro128PlusPlus::from_seed([3, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

    let mut v = vec![0; accesses];

    for i in 0..(accesses / 4) {
        let rng = 0_u32..(size as u32);
        v[i << 2] = rand1.random_range(rng.clone());
        v[(i << 2) + 1] = rand2.random_range(rng.clone());
        v[(i << 2) + 2] = rand3.random_range(rng.clone());
        v[(i << 2) + 3] = rand4.random_range(rng.clone());
    }

    return v;
}

fn main() {
    core_affinity::set_single_core().unwrap();
    let mut rep_tester = RepTester::new().unwrap();

    const SIZE: usize = 512;
    const BYTES_IN_MB: usize = 1024 * 1024;
    let buf = RawAlloc::new(SIZE * BYTES_IN_MB);
    let ptr = buf.as_ptr() as *const u64;

    for i in 1..=SIZE.ilog2() {
        let mbs: u64 = 2_u64.pow(i);
        let len = mbs * (BYTES_IN_MB as u64);
        let access_patterns = packed_access_pattern(len as usize, SIZE * BYTES_IN_MB);
        let name = format!("no prefetching {}MB", mbs);
        let mut no_prefetching_res: u64 = 0;
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            block = {
                unsafe {
                    no_prefetching_res =
                        prefetching::no_prefetching(len, access_patterns.as_ptr(), ptr);
                }
            }
        );

        let mut prefetching_res: u64 = 0;
        let name = format!("prefetching {}MB", mbs);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            block = {
                unsafe {
                    prefetching_res = prefetching::prefetching(len, access_patterns.as_ptr(), ptr);
                }
            }
        );

        assert_eq!(prefetching_res, no_prefetching_res);
    }
}
