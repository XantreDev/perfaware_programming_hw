use std::env;

use haversine_generator::{
    rep_run,
    rep_tester::RepTester,
    write::{RawAlloc, write_backwards, write_linear},
};

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        panic!("argument: [number_of_bytes]")
    }

    let bytes: usize = args.nth(1).unwrap().parse().unwrap();
    let mut rep_tester = RepTester::new().unwrap();

    loop {
        for i in 0..4 {
            let with_malloc = i % 2 == 1;
            let forwards = i / 2 == 0;

            let mut ptr = RawAlloc::new(bytes);
            rep_run!(
                rep_tester,
                name = match (with_malloc, forwards) {
                    (false, true) => "WriteBytesForwards",
                    (true, true) => "malloc + WriteBytesForwards",
                    (false, false) => "WriteBytesBackwards",
                    (true, false) => "malloc + WriteBytesBackwards",
                },
                len = bytes,
                before = {
                    if with_malloc {
                        ptr = RawAlloc::new(bytes)
                    }
                    let slice = ptr.as_u8_slice_mut();
                },
                block = {
                    if forwards {
                        write_linear(slice, 0, slice.len());
                    } else {
                        write_backwards(slice, 0, slice.len());
                    }
                },
                check =
                    { slice.iter().fold(0 as u64, |acc, it| acc + *it as u64) > bytes as u64 - 2 }
            );
        }
    }
}
