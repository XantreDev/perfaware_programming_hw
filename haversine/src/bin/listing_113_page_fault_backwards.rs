use std::{env, fs::File, io::Write};

use haversine_generator::{
    rep_tester::page_faults,
    write::{RawAlloc, write_backwards},
};

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        panic!("argument: [number_of_pages]")
    }

    let pages: usize = args.nth(1).unwrap().parse().unwrap();
    static PAGE: usize = 4 * 1024;
    let len = pages * PAGE;

    let mut faults = vec![0; pages + 1];

    println!("len {}", len);
    let root_faults = page_faults();

    let ptr = RawAlloc::new(len);
    let slice = ptr.as_u8_slice_mut();

    for i in 0..pages {
        write_backwards(slice, i, i * PAGE);

        faults[i] = page_faults() - root_faults;
    }

    println!("total faults {}", page_faults() - root_faults);

    const PAGES: &'static str = &"Total pages;Current page;Actual\n";
    let mut csv = String::with_capacity(PAGES.len() + (4 + 2) * 3);
    csv.push_str(PAGES);

    for i in 0..pages {
        use std::fmt::Write;
        writeln!(&mut csv, "{};{};{}", pages, i, faults[i]).unwrap();
    }

    File::create("./listing_113.csv")
        .unwrap()
        .write_all(&csv.as_bytes())
        .unwrap();
}
