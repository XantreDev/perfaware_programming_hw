use std::env::args;

use asm::load_store_ports;
use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

type LoadPtr = unsafe extern "C" fn(iterations: u64, addr: *mut u64);
struct LoadPortKind {
    name: &'static str,
    ptr: LoadPtr,
}

type StorePtr = unsafe extern "C" fn(iterations: u64, *mut u64, *mut u64, *mut u64, *mut u64);
struct StorePortKind {
    name: &'static str,
    ptr: StorePtr,
}

impl LoadPortKind {
    fn new(name: &'static str, ptr: LoadPtr) -> Self {
        Self { name, ptr }
    }
}

impl StorePortKind {
    fn new(name: &'static str, ptr: StorePtr) -> Self {
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
    assert!(op == "load" || op == "anomaly" || op == "store");
    assert!(loops > 0 && loops < 50_000_000);

    core_affinity::set_single_core().unwrap();
    let mut tester = RepTester::new().unwrap();

    if op == "load" {
        loop {
            let loads = [
                LoadPortKind::new("read_1", load_store_ports::read_1),
                LoadPortKind::new("read_2", load_store_ports::read_2),
                LoadPortKind::new("read_3", load_store_ports::read_3),
                LoadPortKind::new("read_4", load_store_ports::read_4),
            ];
            let mut mem = Box::new(0u64);

            for load_kind in loads {
                rep_run!(
                    tester,
                    name = load_kind.name,
                    len = loops,
                    block = {
                        unsafe {
                            let ptr = load_kind.ptr;
                            ptr(loops, &mut *mem);
                        }
                    }
                );
            }
        }
    } else if op == "anomaly" {
        loop {
            let loads = [
                LoadPortKind::new("read_1x2", load_store_ports::read_1x2),
                LoadPortKind::new("read_8x2", load_store_ports::read_8x2),
            ];

            let mut mem = Box::new(0u64);

            for load_kind in loads {
                rep_run!(
                    tester,
                    name = load_kind.name,
                    len = loops,
                    block = {
                        unsafe {
                            let ptr = load_kind.ptr;
                            ptr(loops, &mut *mem);
                        }
                    }
                );
            }
        }
    } else {
        loop {
            let stores = [
                StorePortKind::new("write_1", load_store_ports::write_1),
                StorePortKind::new("write_2", load_store_ports::write_2),
                StorePortKind::new("write_3", load_store_ports::write_3),
                StorePortKind::new("write_4", load_store_ports::write_4),
            ];

            let mut mem = Box::new([0u64; 4]);

            for store_kind in stores {
                rep_run!(
                    tester,
                    name = store_kind.name,
                    len = loops,
                    block = {
                        unsafe {
                            let ptr = store_kind.ptr;
                            ptr(loops, &mut mem[0], &mut mem[1], &mut mem[2], &mut mem[3]);
                        }
                    }
                );
            }
        }
    }
}
