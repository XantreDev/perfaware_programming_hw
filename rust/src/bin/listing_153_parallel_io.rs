// inspired by
// https://www.computerenhance.com/p/overlapping-file-reads-with-computation
//
use std::{
    env::args,
    fs::File,
    io::Read,
    os::unix::fs::{FileExt, MetadataExt},
    slice::from_raw_parts,
    sync::{
        Arc, RwLock,
        atomic::AtomicU8,
        mpsc::{self, SendError},
    },
    thread,
};

use haversine_generator::{
    SetupError, core_affinity,
    csv_exporter::{CsvColRef, CsvData, PerfEntry},
    pretty_print_u64, rep_run,
    rep_tester::{MeasurementKind, RepTester},
    write::RawAlloc,
};

fn allocate_read_file_and_copy_chunked(
    file: &File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
    let _ = default_cpuset;
    let buf_alloc = RawAlloc::new(chunk_buf_size);
    let buf = buf_alloc.as_u8_slice_mut();

    let mut remaining = total_size as i64;

    while (remaining - chunk_buf_size as i64) >= 0 {
        file.read_exact_at(buf, total_size - remaining as u64)
            .unwrap();
        remaining -= chunk_buf_size as i64;
    }

    if remaining > 0 {
        file.read_exact_at(
            &mut buf[0..(remaining as usize)],
            total_size - remaining as u64,
        )
        .unwrap();
    }

    0
}

struct TestCase {
    name: &'static str,
    col: CsvColRef,
    func: fn(
        file: &File,
        total_size: u64,
        chunk_buf_size: usize,
        default_cpuset: libc::cpu_set_t,
    ) -> u64,
    is_real_sum: bool,
}

fn add_test_case(
    csv_data: &mut CsvData,
    name: &'static str,
    func: fn(
        file: &File,
        total_size: u64,
        chunk_buf_size: usize,
        default_cpuset: libc::cpu_set_t,
    ) -> u64,
    is_read_sum: bool,
) -> TestCase {
    let col_ref = csv_data.col(name);

    TestCase {
        name,
        col: col_ref,
        func: func,
        is_real_sum: is_read_sum,
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

    total1
        .wrapping_add(total2)
        .wrapping_add(total3)
        .wrapping_add(total4)
}

fn allocate_and_compute(
    file: &File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
    let _ = default_cpuset;
    let alloc = RawAlloc::new(chunk_buf_size);
    let buf = alloc.as_u8_slice_mut();

    let mut remaining = total_size as i64;

    let mut total: u64 = 0;
    while (remaining - chunk_buf_size as i64) >= 0 {
        file.read_exact_at(buf, total_size - remaining as u64)
            .unwrap();
        remaining -= chunk_buf_size as i64;

        total = total.wrapping_add(sum_as_u64(buf));
    }

    if remaining > 0 {
        file.read_exact_at(
            &mut buf[0..(remaining as usize)],
            total_size - remaining as u64,
        )
        .unwrap();
        buf[(remaining as usize)..].fill(0);

        total = total.wrapping_add(sum_as_u64(buf));
    }

    total
}

fn parallel_allocate_and_compute(
    file: &File,
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
                    file.read_exact_at(
                        &mut writable.buf[0..(remaining as usize)],
                        total_size - remaining as u64,
                    )
                    .unwrap();
                    writable.buf[(remaining as usize)..].fill(0);

                    remaining = 0;
                } else {
                    file.read_exact_at(writable.buf, total_size - remaining as u64)
                        .unwrap();
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

fn two_parallel_reads(
    file: &File,
    total_size: u64,
    chunk_buf_size: usize,
    default_cpuset: libc::cpu_set_t,
) -> u64 {
    if total_size <= chunk_buf_size as u64 {
        return allocate_and_compute(file, total_size, chunk_buf_size, default_cpuset);
    }
    let allocs = [RawAlloc::new(chunk_buf_size), RawAlloc::new(chunk_buf_size)];
    let workers = allocs.len();
    assert!(workers == 2);

    let bufs = allocs.each_ref().map(|it| it.as_u8_slice_mut());

    #[derive(Clone)]
    enum Message {
        Count(u64),
        Error(String),
        End,
    }
    let (c_sender, c_reciever) = mpsc::channel::<Message>();

    let total = thread::scope(|s| {
        let logical_chunks_count =
            (total_size + (chunk_buf_size as u64 - 1)) / chunk_buf_size as u64;

        for (i, mut_buf) in bufs.into_iter().enumerate() {
            let c_sender = c_sender.clone();
            let file = file.try_clone().unwrap();
            s.spawn(move || -> Result<(), SendError<Message>> {
                core_affinity::set_core_affinity(&default_cpuset).unwrap();

                let chunks_per_worker = (logical_chunks_count + 1) / 2;
                let offset_chunks = chunks_per_worker * i as u64;
                let expected_chunks = chunks_per_worker * (i + 1) as u64;

                // let bytes_per_worker = chunk_buf_size as u64 * chunks_per_worker;

                let mut offset = offset_chunks * chunk_buf_size as u64;
                let expected = (expected_chunks * chunk_buf_size as u64).min(total_size);

                let mut remaining = (expected - offset) as i64;
                // println!("{}", remaining);

                loop {
                    if remaining <= 0 {
                        break;
                    }
                    let read_result = if remaining < chunk_buf_size as i64 {
                        let res = file.read_exact_at(&mut mut_buf[0..(remaining as usize)], offset);

                        mut_buf[(remaining as usize)..].fill(0);

                        remaining = 0;

                        res
                    } else {
                        let res = file.read_exact_at(mut_buf, offset);
                        remaining -= chunk_buf_size as i64;

                        res
                    };

                    match read_result {
                        Ok(_) => {}
                        Err(err) => {
                            c_sender.send(Message::Error(err.to_string()))?;
                            return Ok(());
                        }
                    };

                    offset += chunk_buf_size as u64;

                    c_sender.send(Message::Count(sum_as_u64(&mut_buf)))?;
                }

                c_sender.send(Message::End)?;
                Ok(())
            });
        }

        let mut total: u64 = 0;
        let mut ends: usize = 0;
        for item in c_reciever {
            match item {
                Message::Error(err) => return Err(err),
                Message::Count(count) => {
                    total = total.wrapping_add(count);
                }
                Message::End => {
                    ends += 1;
                    if ends == workers {
                        break;
                    }
                }
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
        false,
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

    let two_workers = add_test_case(
        &mut csv_data,
        "IO Read + Compute x2",
        two_parallel_reads,
        true,
    );

    let cases = [
        file_read_case,
        sequential_compute_case,
        parallel_compute_case,
        two_workers,
    ];
    let mut file = File::open(&file_path).unwrap();

    let size = file.metadata().unwrap().size();
    assert!(size <= usize::MAX as u64);

    let expected_result = allocate_and_compute(&file, size, 1 << 15, orignal_cpuset);
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
