// hw: https://www.computerenhance.com/p/2x-faster-file-reads

use std::{env::args, fs::File, io::Read, os::unix::fs::MetadataExt};

use haversine_generator::{
    csv_exporter::{CsvColRef, CsvData, PerfEntry},
    rep_run,
    rep_tester::MeasurementKind,
    setup_rep_test,
    write::RawAlloc,
};

fn fault_all(buf: &mut [u8]) {
    for i in (0..buf.len()).step_by(4096) {
        buf[i] = 0;
    }
}

fn allocate_chunk_and_fault(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    scratch: &mut [u8],
) {
    let _ = (file, scratch, total_size);

    let buf_alloc = RawAlloc::new(chunk_buf_size);
    fault_all(buf_alloc.as_u8_slice_mut());
}
fn allocate_and_copy_chunked(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    scratch: &mut [u8],
) {
    let _ = (file, total_size);

    let buf_alloc = RawAlloc::new(chunk_buf_size);
    let buf = buf_alloc.as_u8_slice_mut();

    let mut source_offset = 0;
    let mut remaining = scratch.len() as i64;

    while (remaining - chunk_buf_size as i64) >= 0 {
        buf.copy_from_slice(&scratch[source_offset..(source_offset + chunk_buf_size)]);

        source_offset += chunk_buf_size;
        remaining -= chunk_buf_size as i64;
    }
    if remaining > 0 {
        buf[0..remaining as usize].copy_from_slice(&scratch[source_offset..scratch.len()]);
    }
}

fn allocate_read_file_and_copy_chunked(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    scratch: &mut [u8],
) {
    let _ = scratch;
    let buf_alloc = RawAlloc::new(chunk_buf_size);
    let buf = buf_alloc.as_u8_slice_mut();

    let mut remaining = total_size as i64;

    while (remaining - chunk_buf_size as i64) >= 0 {
        file.read_exact(buf).unwrap();
        remaining -= chunk_buf_size as i64;
    }

    if remaining > 0 {
        file.read_exact(&mut buf[0..(remaining as usize)]).unwrap();
    }
}

struct TestCase {
    name: &'static str,
    col: CsvColRef,
    func: fn(file: &mut File, total_size: u64, chunk_buf_size: usize, scratch: &mut [u8]),
    uses_file: bool,
    only_buf_len: bool,
}

fn add_test_case(
    csv_data: &mut CsvData,
    name: &'static str,
    func: fn(file: &mut File, total_size: u64, chunk_buf_size: usize, scratch: &mut [u8]),
    uses_file: bool,
    only_buf_len: bool,
) -> TestCase {
    let col_ref = csv_data.col(name);

    TestCase {
        name,
        col: col_ref,
        func: func,
        uses_file: uses_file,
        only_buf_len: only_buf_len,
    }
}

fn main() {
    let file_path = {
        let file_path = args().nth(1).expect("must pass file_path");

        file_path
    };
    let mut rep_tester = setup_rep_test().unwrap();
    let mut csv_data = CsvData::new();

    let fault_all_case = add_test_case(
        &mut csv_data,
        "Fault Buffer",
        allocate_chunk_and_fault,
        false,
        true,
    );
    let copy_case = add_test_case(
        &mut csv_data,
        "Chunked Copy",
        allocate_and_copy_chunked,
        false,
        false,
    );
    let file_read_case = add_test_case(
        &mut csv_data,
        "Chunked IO Read",
        allocate_read_file_and_copy_chunked,
        true,
        false,
    );

    let cases = [fault_all_case, copy_case, file_read_case];
    let mut file = File::open(&file_path).unwrap();

    let size = file.metadata().unwrap().size();
    assert!(size <= usize::MAX as u64);

    let scratch_alloc = RawAlloc::new(size as usize);
    let scratch = scratch_alloc.as_u8_slice_mut();

    for chunk_i in 15..=30 {
        let chunk_len = 1 << chunk_i;
        for case in &cases {
            rep_run!(
                rep_tester,
                name = case.name,
                len = if case.only_buf_len {
                    chunk_len as u64
                } else {
                    size
                },
                before = {
                    if case.uses_file {
                        file = File::open(&file_path).unwrap();
                    }
                },
                block = {
                    let func = case.func;
                    func(&mut file, size, chunk_len, scratch);
                }
            );

            csv_data.row(
                case.col,
                PerfEntry {
                    size: chunk_len as u64,
                    throughput: rep_tester
                        .measurement(MeasurementKind::Best)
                        .throughput_mb(),
                },
            );
        }
    }

    println!("{}", csv_data.export().unwrap_or_default());
}
