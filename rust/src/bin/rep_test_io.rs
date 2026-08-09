use std::{fs::File, io::Read, mem::MaybeUninit, os::fd::AsRawFd, path::Path, process::exit};

use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester, write::RawAlloc};

fn main() {
    use std::env;

    core_affinity::set_single_core().unwrap();
    let mut args = env::args();
    if args.len() < 2 {
        println!("possible args [test_data.json]");
        exit(1);
    }

    let _arg = args.nth(1).unwrap();
    let test_data_path = Path::new(&_arg);

    let file = File::open(test_data_path).expect("file path cannot be open");

    let meta = file.metadata().expect("metadata must exist");

    let mut rep_tester = RepTester::new().unwrap();

    let mut json = String::with_capacity(meta.len() as usize);

    let with_malloc = false;
    loop {
        let mut value: i64 = 0;
        let i64_size = i64::BITS as usize / 8;
        let size: usize = 700 * 1024 * 1024 / i64_size;
        rep_run!(
            rep_tester,
            name = "page_faults_check",
            len = size * i64_size,
            before = {
                let seed = rand::random::<i64>();
                let buf = RawAlloc::new(size);
                let arr = buf.as_u64_slice_mut();
                // let mut arr: Box<[MaybeUninit<i64>]> = Box::new_uninit_slice(size);
            },
            block = {
                for i in 0..size / 4 {
                    let i = i * 4;
                    arr[i] = (seed * (i as i64 + 1) * 8) as u64;
                    arr[i + 1] = (seed * ((i + 1) as i64 + 1) * 8) as u64;
                    arr[i + 2] = (seed * ((i + 2) as i64 + 1) * 8) as u64;
                    arr[i + 3] = (seed * ((i + 3) as i64 + 1) * 8) as u64;
                }
            },
            check = { arr.len() == size as usize },
            after_run = {
                value += arr.iter().fold(0, |acc, it| acc + *it as i64);
            }
        );
        println!("res {}", value);
        print!("\x1b[1A\x1b[2K");

        if with_malloc {
            rep_run!(
                rep_tester,
                name = "File::read_to_string + reusable string",
                len = meta.len(),
                before = {
                    json.clear();
                    let mut file = File::open(test_data_path).unwrap();
                },
                block = {
                    file.read_to_string(&mut json).unwrap();
                },
                check = { json.len() == meta.len() as usize },
            );

            rep_run!(
                rep_tester,
                name = "File::read_to_string + String::with_capacity",
                len = meta.len(),
                before = {
                    let mut file = File::open(test_data_path).unwrap();
                    let mut json = String::with_capacity(file.metadata().unwrap().len() as usize);
                },
                block = {
                    file.read_to_string(&mut json).unwrap();
                },
                check = { json.len() == meta.len() as usize },
            );

            rep_run!(
                rep_tester,
                name = "File::read",
                len = meta.len(),
                before = {
                    let mut file = File::open(test_data_path).unwrap();
                },
                block = {
                    let mut json_arr = Vec::with_capacity((meta.len() + 1) as usize);
                    file.read_to_end(&mut json_arr).unwrap();
                },
                check = { json_arr.len() == meta.len() as usize },
            );
        }

        // terrible
        // rep_run!(
        //     rep_tester,
        //     name = "loop { File::read } + malloc (4K)",
        //     len = meta.len(),
        //     before = {
        //         let json = RawAlloc::new(meta.len() as usize);
        //         unsafe { libc::madvise(json.as_mut_ptr(), json.size(), libc::MADV_NOHUGEPAGE) };

        //         let mut file = File::open(test_data_path).unwrap();
        //         let buf = json.as_u8_slice_mut();
        //     },
        //     block = {
        //         let mut read = 0;
        //         loop {
        //             let cur_read = file.read(&mut buf[read..]).unwrap();
        //             read += cur_read;
        //             if cur_read == 0 {
        //                 break;
        //             }
        //         }
        //     },
        //     check = { buf.len() == meta.len() as usize },
        // );

        rep_run!(
            rep_tester,
            name = "File::read_exact + mmap(aligned)",
            len = meta.len(),
            before = {
                let mut file = File::open(test_data_path).unwrap();
            },
            block = {
                let json = RawAlloc::new(round_up_to_2mb(meta.len() as usize));
                let buf = &mut json.as_u8_slice_mut()[0..(meta.len() as usize)];
                file.read_exact(buf).unwrap();
            },
            check = { buf[buf.len() - 2] != 0 },
        );

        #[cfg(target_os = "linux")]
        rep_run!(
            rep_tester,
            name = "libc::read + mmap(aligned)",
            len = meta.len(),
            before = {
                let file = File::open(test_data_path).unwrap();
                let json = RawAlloc::new(round_up_to_2mb(meta.len() as usize));
            },
            block = {
                let result =
                    unsafe { libc::read(file.as_raw_fd(), json.as_mut_ptr(), meta.len() as usize) };
            },
            check = { result as u64 == meta.len() },
        );

        #[cfg(target_os = "linux")]
        rep_run!(
            rep_tester,
            name = "libc::read + mmap(aligned)",
            len = meta.len(),
            before = {
                let file = File::open(test_data_path).unwrap();
            },
            block = {
                let json = RawAlloc::new(round_up_to_2mb(meta.len() as usize));
                let result =
                    unsafe { libc::read(file.as_raw_fd(), json.as_mut_ptr(), meta.len() as usize) };
                let json = &mut json.as_u8_slice_mut()[..(result as usize)];
                let result = str::from_utf8(json).unwrap();
            },
            check = { result.len() as u64 == meta.len() },
        );
    }
}

fn round_up_to_2mb(x: usize) -> usize {
    const TWO_MB: usize = 1 << 21;
    (x + TWO_MB - 1) & !(TWO_MB - 1)
}
