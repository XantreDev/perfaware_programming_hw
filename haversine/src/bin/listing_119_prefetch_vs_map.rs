use std::env::args;

use haversine_generator::{
    rep_run,
    rep_tester::RepTester,
    write::{RawAlloc, write_linear},
};

#[inline]
fn prefault(slice: &mut [u8]) {
    let PAGE = 1024 * 4;
    for i in (0..slice.len()).step_by(PAGE) {
        slice[i] = 1;
        slice[i] = 0;
    }
}

fn main() {
    let size: u16 = args()
        .nth(1)
        .expect("[size MBs]")
        .parse()
        .expect("[size MBs]");
    let mut rep_tester = RepTester::new().unwrap();
    let bytes = size as usize * 1024 * 1024;

    println!("Bench for {}", bytes);

    loop {
        rep_run!(
            rep_tester,
            name = "fault",
            len = bytes,
            before = {
                let array = RawAlloc::new(bytes);
                let slice = array.as_u8_slice_mut();
            },
            block = {
                write_linear(slice, 0, slice.len());
            },
            check = { slice.iter().fold(0 as u64, |acc, it| acc + *it as u64) > bytes as u64 - 2 }
        );

        rep_run!(
            rep_tester,
            name = "pre-fault",
            len = bytes,
            before = {
                let array = RawAlloc::new(bytes);
                let slice = array.as_u8_slice_mut();
            },
            block = {
                prefault(slice);
                write_linear(slice, 0, slice.len());
            },
            check = { slice.iter().fold(0 as u64, |acc, it| acc + *it as u64) > bytes as u64 - 2 }
        );
    }
}
