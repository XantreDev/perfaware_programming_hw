use std::{
    mem,
    ptr::{null, null_mut},
};

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
    // let buffer = memory.as_u8_slice_mut();
    const STEP_BITS: usize = 3;

    let mut rep_tester = RepTester::new().unwrap();
    for i in 10..30 {
        for step in 0..(1 << STEP_BITS) {
            let memory_chunk_size = (1 << i) | (step << (i - STEP_BITS));

            println!("{:#010b}", memory_chunk_size);
            let name: String;
            if (i - 3) >= 20 {
                name = format!("cache_{}MB", memory_chunk_size >> 20);
            } else if (i - 3) >= 10 {
                name = format!("cache_{}kB", memory_chunk_size >> 10);
            } else {
                name = format!("cache_{}B", memory_chunk_size);
            }
            let iterations = (BUF_SIZE as f64 / memory_chunk_size as f64).ceil() as u64;
            let adjusted_len = iterations * memory_chunk_size;
            println!("it={}, len={}", iterations, adjusted_len);

            rep_run!(
                rep_tester,
                name = &name,
                len = adjusted_len,
                block = {
                    unsafe {
                        // let ptr = memory.as_u8_mut_ptr();

                        // let mem = Box::new([0; BUF_SIZE]);
                        // let value = *(ptr.wrapping_add(1024));
                        println!("calling");
                        asm::non_bin_cache::test_cache_non_bin(
                            0,
                            0,
                            // mem
                            // iterations,
                            // memory_chunk_size / (asm::non_bin_cache::READ_SIZE as u64),
                            null_mut::<u64>(),
                            // mem.as_mut_ptr(),
                        );

                        println!("after call");
                    }
                }
            );
        }
    }
}
