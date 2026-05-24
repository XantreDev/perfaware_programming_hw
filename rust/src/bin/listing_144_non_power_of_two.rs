use std::{
    env::args,
    io::{Write, stdout},
};

use haversine_generator::{
    core_affinity, rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

struct CachePerformance {
    size: u64,
    throughput: f64,
}

fn csv_of(items: &[CachePerformance]) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(32 * (1 + items.len()));

    writeln!(bytes, "Size,Throughput").unwrap();
    for item in items {
        writeln!(bytes, "{},{}", item.size, item.throughput).unwrap();
    }

    unsafe { String::from_utf8_unchecked(bytes) }
}

fn main() {
    core_affinity::set_single_core().unwrap();

    let args: Vec<String> = args().skip(1).collect();
    assert!(args.len() == 1);
    let to_csv = {
        let val = &args[0];
        if val == "csv" {
            true
        } else if val == "info" {
            false
        } else {
            panic!("output must be 'csv' or 'info'");
        }
    };

    const BUF_BITS: usize = 30;
    const BUF_SIZE: usize = 1 << BUF_BITS;
    let memory = RawAlloc::new(BUF_SIZE);
    // let buffer = memory.as_u8_slice_mut();
    const STEP_BITS: usize = 3;

    let mut i: u8 = 0;
    for item in memory.as_u8_slice_mut() {
        *item = i;
        i = u8::wrapping_add(i, 1);
    }

    let mut rep_tester = RepTester::new().unwrap();
    rep_tester.print = !to_csv;
    let mut table: Vec<CachePerformance> = Vec::with_capacity(1024);
    for i in 14..27 {
        for step in 0..(1 << STEP_BITS) {
            let memory_chunk_size = (1 << i) | (step << (i - STEP_BITS));

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
            // println!(
            //     "it={}, len={}, inner_iter={}",
            //     iterations,
            //     adjusted_len,
            //     memory_chunk_size / (asm::non_bin_cache::READ_SIZE as u64),
            // );

            rep_run!(
                rep_tester,
                name = &name,
                len = adjusted_len,
                block = {
                    unsafe {
                        let ptr = memory.as_mut_ptr() as *mut u64;

                        asm::non_bin_cache::test_cache_non_bin(
                            iterations,
                            memory_chunk_size / (asm::non_bin_cache::READ_SIZE as u64),
                            ptr,
                        );
                    }
                }
            );
            if to_csv {
                let measurement = rep_tester.measurement(rep_tester::MeasurementKind::Best);

                table.push(CachePerformance {
                    size: memory_chunk_size,
                    throughput: measurement.throughput_mb(),
                });
            }
        }
    }
    if to_csv {
        let out = csv_of(&table);
        write!(stdout(), "{}", out).unwrap();
    }
}
