use std::{
    env::args,
    io::{Write, stdout},
};

use haversine_generator::{
    pretty_print_u64, profiling_labels,
    rep_tester::page_faults,
    simple_profiler,
    time::TimeMeasurer,
    with_label, with_profiling,
    write::{RawAlloc, write_linear},
};

struct ParsedPointer {
    offset: u32,
    table_index: u32,
    directory_index: u32,
    directory_ptr_index: u32,
    pml4_index: u32,
    prefix: u32,
}

fn parse_pointer(ptr: *const libc::c_void) -> ParsedPointer {
    let ptr = ptr as u64;

    fn read_bits(value: u64, right_offset: u64, bits: u64) -> u64 {
        let mask = (1 << bits) - 1;
        (value & (mask << right_offset)) >> right_offset
    }

    ParsedPointer {
        offset: read_bits(ptr, 0, 12) as u32,
        table_index: read_bits(ptr, 12, 9) as u32,
        directory_index: read_bits(ptr, 12 + 9, 9) as u32,
        directory_ptr_index: read_bits(ptr, 12 + 9 * 2, 9) as u32,
        pml4_index: read_bits(ptr, 12 + 9 * 3, 9) as u32,
        prefix: read_bits(ptr, 12 + 9 * 4, 16) as u32,
    }
}

profiling_labels! {
    enum Labels {
        Write = 1,
    }
}

fn main() {
    let tries = 10;
    let mut ptrs = Vec::with_capacity(tries);
    let mut out = stdout();
    let size = 1024 * 1024 * args().nth(1).unwrap().parse::<usize>().unwrap();

    let measurer = TimeMeasurer::init().unwrap();
    writeln!(
        &mut out,
        "---\nAllocating {}MB buffers\n---\n",
        size as f64 / 1024.0 / 1024.0
    )
    .unwrap();
    for _ in 0..tries {
        let ptr = RawAlloc::new(size);

        let parsed_ptr = parse_pointer(ptr.as_ptr());
        writeln!(
            &mut out,
            r"Prefix | PML4 | DirPtr | DirIdx | TableIdx | Offset
{: ^6} | {: ^4} | {: ^6} | {: ^6} | {: ^8} | {: ^6}
{:064b}",
            parsed_ptr.prefix,
            parsed_ptr.pml4_index,
            parsed_ptr.directory_ptr_index,
            parsed_ptr.directory_index,
            parsed_ptr.table_index,
            parsed_ptr.offset,
            ptr.as_ptr() as u64
        )
        .unwrap();

        let clocks_before = measurer.clocks_now();
        let faults_before = page_faults();
        with_profiling! { Labels =>
            with_label! { Labels::Write where bytes=ptr.as_u8_slice_mut().len() =>
                write_linear(ptr.as_u8_slice_mut(), 0, ptr.as_u8_slice_mut().len());
            }
        };
        writeln!(out, "faults {}\n", page_faults() - faults_before).unwrap();
        // writeln!(
        //     out,
        //     "Clocks consumed: {}; Page faults: {}\n",
        //     pretty_print_u64(measurer.clocks_now() - clocks_before),
        //     page_faults() - faults_before
        // )
        // .unwrap();
        out.flush().unwrap();

        ptrs.push(ptr);
    }
}
