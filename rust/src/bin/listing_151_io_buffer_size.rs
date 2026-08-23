// hw: https://www.computerenhance.com/p/2x-faster-file-reads

use std::{sync::mpsc, thread};

use haversine_generator::{rep_run, setup_rep_test, write::RawAlloc};

fn fault_all(buf: &mut [u8]) {
    for i in (0..buf.len()).step_by(4096) {
        buf[i] = 0;
    }
}

fn main() {
    let mut rep_tester = setup_rep_test().unwrap();
    // let (tx, rx) = mpsc::channel();
    let src_alloc = RawAlloc::new(1 << 30);
    let src = src_alloc.as_u8_slice_mut();

    fault_all(src);
    for i in 14..=30 {
        rep_run!(
            rep_tester,
            name = "Fault All",
            len = src.len(),
            before = {
                let buf_alloc = RawAlloc::new(1 << i);
                let buf = buf_alloc.as_u8_slice_mut();
            },
            block = {
                fault_all(buf);
            }
        );
    }
}
