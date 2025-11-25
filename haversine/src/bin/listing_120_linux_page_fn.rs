use std::{
    env::args,
    io::{Write, stdout},
};

use haversine_generator::{
    pointer::{parse_ptr, parse_raw_pointer},
    rep_tester::page_faults,
    write::RawAlloc,
};

#[derive(Debug)]
struct PageSwitchEntry {
    iteration: u32,
    faults: u32,
    table_idx: u32,
    dir_idx: u32,
}

const PAGE: u64 = 4 * 1024;
fn main() {
    let size: u16 = args()
        .nth(1)
        .expect("[size MBs]")
        .parse()
        .expect("[size MBs]");
    let bytes = size as usize * 1024 * 1024;

    println!("Page limiting for {}MB", bytes / 1024 / 1024);

    let ptr = RawAlloc::new(bytes);

    let slice = ptr.as_u8_slice_mut();

    let mut entries = Vec::with_capacity(bytes / 4 / 1024 + 1); // String::with_capacity((bytes / 4 / 1024 + 1) * 20);

    let start_pages = page_faults();
    let mut write_fauts = 0;
    let mut prev_delta = 0;

    for i in 0..bytes {
        slice[i] = (i * 10) as u8;
        let parsed_ptr = parse_ptr(ptr.as_u8_ptr() as u64 + i as u64);

        let delta = (page_faults()) - (start_pages + write_fauts);
        if prev_delta != delta {
            // use std::fmt::Write;
            let write_delta = page_faults();
            entries.push(PageSwitchEntry {
                iteration: i as u32,
                faults: delta as u32,
                table_idx: parsed_ptr.table_index,
                dir_idx: parsed_ptr.directory_index,
            });
            write_fauts += page_faults() - write_delta;
            prev_delta = delta;
        }
    }

    let mut step = 0;
    let mut out = stdout();
    for i in 1..entries.len() {
        let next_step = entries[i].iteration - entries[i - 1].iteration;

        if next_step != step {
            use std::io::Write;
            writeln!(
                &mut out,
                "{} -> {}KB at (page: {}, table_idx: {}, faults_per_switch: {})",
                step,
                next_step / 1024,
                entries[i - 1].iteration as u64 / PAGE,
                entries[i - 1].table_idx,
                entries[i].faults - entries[i - 1].faults
            )
            .unwrap();
            step = next_step;
        }
    }

    out.flush().unwrap();
}
