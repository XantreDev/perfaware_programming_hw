use std::{
    fs::{self, File},
    io::Read,
    mem::MaybeUninit,
    path::Path,
    process::exit,
};

use haversine_generator::{rep_run, rep_tester::RepTester, write::RawAlloc};

struct TestFn<'a> {
    name: &'static str,
    args: &'a String,
    exepcted_size: u64,
    out: String,
    block: &'a dyn Fn(&String) -> String,
}

fn main() {
    use std::env;

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
    let mut json_arr = Vec::with_capacity((meta.len() + 1) as usize);

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
                let mut arr: Box<[MaybeUninit<i64>]> = Box::new_uninit_slice(size);
            },
            block = {
                for i in 0..size / 4 {
                    let i = i * 4;
                    arr[i] = MaybeUninit::new(seed * (i as i64 + 1) * 8);
                    arr[i + 1] = MaybeUninit::new(seed * ((i + 1) as i64 + 1) * 8);
                    arr[i + 2] = MaybeUninit::new(seed * ((i + 2) as i64 + 1) * 8);
                    arr[i + 3] = MaybeUninit::new(seed * ((i + 3) as i64 + 1) * 8);
                }
            },
            check = { arr.len() == size as usize },
            after_run = {
                value += arr
                    .iter()
                    .fold(0, |acc, it| acc + unsafe { it.assume_init() });
            }
        );
        println!("res {}", value);
        print!("\x1b[1A\x1b[2K");

        rep_run!(
            rep_tester,
            name = "File::read_to_string",
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
            name = "File::read_to_string + malloc",
            len = meta.len(),
            before = {
                let mut json = String::new();
                let mut file = File::open(test_data_path).unwrap();
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
                json_arr.clear();
                let mut file = File::open(test_data_path).unwrap();
            },
            block = {
                file.read_to_end(&mut json_arr).unwrap();
            },
            check = { json_arr.len() == meta.len() as usize },
        );

        rep_run!(
            rep_tester,
            name = "loop { File::read } + malloc (4K)",
            len = meta.len(),
            before = {
                let json = RawAlloc::new(meta.len() as usize);
                unsafe { libc::madvise(json.as_mut_ptr(), json.size(), libc::MADV_NOHUGEPAGE) };

                let mut file = File::open(test_data_path).unwrap();
                let buf = json.as_u8_slice_mut();
            },
            block = {
                let mut read = 0;
                loop {
                    let cur_read = file.read(&mut buf[read..]).unwrap();
                    read += cur_read;
                    if cur_read == 0 {
                        break;
                    }
                }
            },
            check = { buf.len() == meta.len() as usize },
        );

        rep_run!(
            rep_tester,
            name = "File::read_exact + malloc aligned",
            len = meta.len(),
            before = {
                let json = RawAlloc::new(round_up_to_2mb(meta.len() as usize));

                let mut file = File::open(test_data_path).unwrap();
                let buf = &mut json.as_u8_slice_mut()[0..(meta.len() as usize)];
            },
            block = {
                file.read_exact(buf).unwrap();
            },
            check = { buf[buf.len() - 2] != 0 },
        );

        rep_run!(
            rep_tester,
            name = "File::read_exact + malloc (auto)",
            len = meta.len(),
            before = {
                let json = RawAlloc::new(meta.len() as usize);

                let mut file = File::open(test_data_path).unwrap();
                let buf = json.as_u8_slice_mut();
            },
            block = {
                file.read_exact(buf).unwrap();
            },
            check = { buf.len() == meta.len() as usize },
        );
    }
}

fn round_up_to_2mb(x: usize) -> usize {
    const TWO_MB: usize = 1 << 21;
    (x + TWO_MB - 1) & !(TWO_MB - 1)
}
