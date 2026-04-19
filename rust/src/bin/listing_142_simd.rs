use std::env::args;

use aligned::Aligned;
use asm::simd;
use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

type LoadPtr = unsafe extern "C" fn(iterations: u64, addr: *mut u64);
struct LoadPortKind {
    name: &'static str,
    ptr: LoadPtr,
}

impl LoadPortKind {
    fn new(name: &'static str, ptr: LoadPtr) -> Self {
        Self { name, ptr }
    }
}

fn main() {
    let args = args().skip(1);
    assert!(args.len() == 2);
    let args: Vec<String> = args.collect();
    let op = &args[0];
    let loops: u64 = args[1]
        .replace('_', "")
        .parse()
        .expect("the second arg must be amount of loops");
    assert!(op == "load_lines" || op == "load_width" || op == "write_lines");
    assert!(loops > 0 && loops <= 100_000_000);

    core_affinity::set_single_core().unwrap();
    let mut tester = RepTester::new().unwrap();

    if op == "load_width" {
        // let mut array = [0u64; 256];
        let mut array: Aligned<aligned::A64, [u64; 256]> = Aligned([0u64; 256]);

        loop {
            let ops = [
                LoadPortKind::new("read_4x3", simd::read_4x3),
                LoadPortKind::new("read_8x3", simd::read_8x3),
                LoadPortKind::new("read_16x3", simd::read_16x3),
                // LoadPortKind::new("read_32x2", simd::read_32x2),
                LoadPortKind::new("read_32x3", simd::read_32x3),
                // LoadPortKind::new("read_32x4", simd::read_32x4),
            ];
            for item in ops {
                rep_run!(
                    tester,
                    name = item.name,
                    len = loops,
                    block = {
                        let ptr = item.ptr;
                        unsafe { ptr(loops, array.as_mut_ptr()) }
                    }
                );
            }
        }
    } else if op == "load_lines" {
        // let mut array = [0u64; 256];
        let mut array: Aligned<aligned::A64, [u64; 256]> = Aligned([0u64; 256]);

        loop {
            let ops = [
                LoadPortKind::new("read_same_32x1", simd::read_same_32x1),
                LoadPortKind::new("read_same_32x2", simd::read_same_32x2),
                LoadPortKind::new("read_same_32x3", simd::read_same_32x3),
                LoadPortKind::new("read_same_32x4", simd::read_same_32x4),
            ];
            for item in ops {
                rep_run!(
                    tester,
                    name = item.name,
                    len = loops,
                    block = {
                        let ptr = item.ptr;
                        unsafe { ptr(loops, array.as_mut_ptr()) }
                    }
                );
            }
        }
    } else if op == "write_lines" {
        // let mut array = [0u64; 256];
        let mut array: Aligned<aligned::A64, [u64; 256]> = Aligned([0u64; 256]);
        loop {
            let ops = [
                LoadPortKind::new("write_32x1", simd::write_32x1),
                LoadPortKind::new("write_32x2", simd::write_32x2),
                LoadPortKind::new("write_32x3", simd::write_32x3),
            ];
            for item in ops {
                rep_run!(
                    tester,
                    name = item.name,
                    len = loops,
                    block = {
                        let ptr = item.ptr;
                        unsafe { ptr(loops, array.as_mut_ptr()) }
                    }
                );
            }
        }
    } else {
        panic!("invariant op {}", op);
    }
}
