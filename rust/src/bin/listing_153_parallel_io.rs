// inspired by
// https://www.computerenhance.com/p/overlapping-file-reads-with-computation
//
use std::{
    env::args,
    fs::File,
    io::Read,
    os::unix::fs::{FileExt, MetadataExt},
    slice::from_raw_parts,
    sync::{Arc, RwLock, atomic::AtomicU8, mpsc},
    thread,
};

use haversine_generator::{
    SetupError, core_affinity,
    csv_exporter::{CsvColRef, CsvData, PerfEntry},
    pretty_print_u64, rep_run,
    rep_tester::{MeasurementKind, RepTester},
    setup_rep_test_single_core,
    write::RawAlloc,
};

fn fault_all(buf: &mut [u8]) {
    for i in (0..buf.len()).step_by(4096) {
        buf[i] = 0;
    }
}

fn allocate_read_file_and_copy_chunked(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
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

    0
}

struct TestCase {
    name: &'static str,
    col: CsvColRef,
    func: fn(
        file: &mut File,
        total_size: u64,
        chunk_buf_size: usize,
        default_cpuset: libc::cpu_set_t,
    ) -> u64,
    uses_file: bool,
}

fn add_test_case(
    csv_data: &mut CsvData,
    name: &'static str,
    func: fn(
        file: &mut File,
        total_size: u64,
        chunk_buf_size: usize,
        default_cpuset: libc::cpu_set_t,
    ) -> u64,
    uses_file: bool,
) -> TestCase {
    let col_ref = csv_data.col(name);

    TestCase {
        name,
        col: col_ref,
        func: func,
        uses_file: uses_file,
    }
}

fn sum_as_u64(data: &[u8]) -> u64 {
    const DIVIDER: usize = (u64::BITS / u8::BITS) as usize;
    assert!(data.len() % DIVIDER == 0);

    let mut total1: u64 = 0;
    let mut total2: u64 = 0;
    let mut total3: u64 = 0;
    let mut total4: u64 = 0;

    let slice = unsafe {
        let ptr = data.as_ptr();
        from_raw_parts(ptr as *const u64, data.len() / DIVIDER)
    };

    for i in (0..slice.len()).step_by(4) {
        total1 = total1.wrapping_add(slice[i]);
        total2 = total2.wrapping_add(slice[i + 1]);
        total3 = total3.wrapping_add(slice[i + 2]);
        total4 = total4.wrapping_add(slice[i + 3]);
    }

    total1 + total2 + total3 + total4
}

fn allocate_and_compute(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
    let alloc = RawAlloc::new(chunk_buf_size);
    let buf = alloc.as_u8_slice_mut();

    let mut remaining = total_size as i64;

    let mut total: u64 = 0;
    while (remaining - chunk_buf_size as i64) >= 0 {
        file.read_exact(buf).unwrap();
        remaining -= chunk_buf_size as i64;

        total = total.wrapping_add(sum_as_u64(buf));
    }

    if remaining > 0 {
        file.read_exact(&mut buf[0..(remaining as usize)]).unwrap();
        buf[(remaining as usize)..].fill(0);

        total = total.wrapping_add(sum_as_u64(buf));
    }

    total
}

fn parallel_allocate_and_compute(
    file: &mut File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
    let allocs = [RawAlloc::new(chunk_buf_size), RawAlloc::new(chunk_buf_size)];
    #[derive(PartialEq, Eq)]
    enum BufStatus {
        Free,
        Busy,
    }
    struct BufInner<'a> {
        buf: &'a mut [u8],
        status: BufStatus,
    }
    impl<'a> BufInner<'a> {
        fn new(buf: &'a mut [u8]) -> Self {
            return Self {
                buf,
                status: BufStatus::Free,
            };
        }
    }

    let bufs = allocs
        .each_ref()
        .map(|it| Arc::new(RwLock::new(BufInner::new(it.as_u8_slice_mut()))));
    #[derive(Clone)]
    enum Message {
        ReadyBufIdx(u8),
        Error(String),
        End,
    }
    let (c_sender, c_reciever) = mpsc::channel::<Message>();

    let total = thread::scope(|s| {
        s.spawn(|| {
            core_affinity::set_core_affinity(&default_cpuset).unwrap();
            let mut remaining = total_size as i64;
            let mut idx = 0;

            loop {
                if remaining <= 0 {
                    break;
                }
                let item = &bufs[idx];
                let cur_idx = idx;
                idx = (idx + 1) & 1;
                let Ok(read_result) = item.try_read() else {
                    continue;
                };

                if read_result.status != BufStatus::Free {
                    continue;
                }
                drop(read_result);
                let write_result = item.write();
                let mut writable = match write_result {
                    Ok(inner) => inner,
                    Err(err) => {
                        c_sender.send(Message::Error(err.to_string())).unwrap();
                        return;
                    }
                };
                writable.status = BufStatus::Busy;
                if remaining < chunk_buf_size as i64 {
                    file.read_exact(&mut writable.buf[0..(remaining as usize)])
                        .unwrap();
                    writable.buf[(remaining as usize)..].fill(0);

                    remaining = 0;
                } else {
                    file.read_exact(writable.buf).unwrap();
                    remaining -= chunk_buf_size as i64;
                }
                c_sender.send(Message::ReadyBufIdx(cur_idx as u8)).unwrap();
            }

            c_sender.send(Message::End).unwrap();
        });

        let mut total: u64 = 0;
        for item in c_reciever {
            match item {
                Message::Error(err) => return Err(err),
                Message::ReadyBufIdx(idx) => {
                    let mut result = bufs[idx as usize].write().unwrap();
                    total = total.wrapping_add(sum_as_u64(result.buf));

                    result.status = BufStatus::Free;
                }
                Message::End => break,
            }
        }
        Ok(total)
    })
    .unwrap();

    total
}

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

    let file_read_case = add_test_case(
        &mut csv_data,
        "Chunked IO Read",
        allocate_read_file_and_copy_chunked,
        true,
    );

    let sequential_compute_case = add_test_case(
        &mut csv_data,
        "IO Read + Compute",
        allocate_and_compute,
        true,
    );

    let parallel_compute_case = add_test_case(
        &mut csv_data,
        "IO Read + Compute Thread",
        parallel_allocate_and_compute,
        true,
    );

    let cases = [
        file_read_case,
        sequential_compute_case,
        parallel_compute_case,
    ];
    let mut file = File::open(&file_path).unwrap();

    let size = file.metadata().unwrap().size();
    assert!(size <= usize::MAX as u64);

    for chunk_i in 15..=30 {
        let chunk_len = 1 << chunk_i;
        for case in &cases {
            let name = format!("{} [{}]", case.name, pretty_print_u64(chunk_len as u64));
            rep_run!(
                rep_tester,
                name = &name,
                len = size,
                before = {
                    if case.uses_file {
                        file = File::open(&file_path).unwrap();
                    }
                },
                block = {
                    let func = case.func;
                    func(&mut file, size, chunk_len, orignal_cpuset);
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
