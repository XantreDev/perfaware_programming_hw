// inspired by
// https://www.computerenhance.com/p/overlapping-file-reads-with-computation
//
use std::{env::args, fs::File, os::unix::fs::MetadataExt};

use haversine_generator::{
    SetupError, core_affinity,
    csv_exporter::{CsvColRef, CsvData, PerfEntry},
    io_tests, pretty_print_u64, rep_run,
    rep_tester::{MeasurementKind, RepTester},
};

fn main() {
    let file_path = {
        let file_path = args().nth(1).expect("must pass file_path");

        file_path
    };

    let mut rep_tester = RepTester::new()
        .ok_or_else(|| SetupError {
            message: format!("Failed to create tester"),
        })
        .unwrap();
    let orignal_cpuset = core_affinity::get_core_affinity().unwrap();
    core_affinity::set_single_core().unwrap();

    let mut csv_data = CsvData::new();

    let file_read_case = io_tests::add_test_case(
        &mut csv_data,
        "Chunked IO Read",
        io_tests::allocate_read_file_and_copy_chunked,
        false,
    );

    let sequential_compute_case = io_tests::add_test_case(
        &mut csv_data,
        "IO Read + Compute",
        io_tests::allocate_and_compute,
        true,
    );

    let parallel_compute_case = io_tests::add_test_case(
        &mut csv_data,
        "IO Read + Compute Thread",
        io_tests::parallel_allocate_and_compute,
        true,
    );

    let two_workers = io_tests::add_test_case(
        &mut csv_data,
        "IO Read + Compute x2",
        io_tests::two_parallel_reads,
        true,
    );

    let map = io_tests::add_test_case(
        &mut csv_data,
        "File Map + Compute",
        io_tests::map_and_sum,
        true,
    );

    let cases = [
        file_read_case,
        sequential_compute_case,
        parallel_compute_case,
        two_workers,
        map,
    ];
    let mut file = File::open(&file_path).unwrap();

    let size = file.metadata().unwrap().size();
    assert!(size <= usize::MAX as u64);

    let expected_result = io_tests::allocate_and_compute(&file, size, 1 << 15, orignal_cpuset);
    for chunk_i in 15..=30 {
        let chunk_len = 1 << chunk_i;
        for case in &cases {
            let name = format!("{} [{}]", case.name, pretty_print_u64(chunk_len as u64));
            rep_run!(
                rep_tester,
                name = &name,
                len = size,
                before = {},
                block = {
                    let func = case.func;
                    let result = func(&mut file, size, chunk_len, orignal_cpuset);
                },
                check = { !case.is_real_sum || expected_result == result }
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
