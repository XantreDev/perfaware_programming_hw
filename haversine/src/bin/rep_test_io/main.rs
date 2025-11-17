use std::{
    fs::{self, File},
    io::Read,
    mem::MaybeUninit,
    path::Path,
    process::exit,
};

use haversine_generator::rep_tester::RepTester;

struct TestFn<'a> {
    name: &'static str,
    args: &'a String,
    exepcted_size: u64,
    out: String,
    block: &'a dyn Fn(&String) -> String,
}

macro_rules! rep_run {
    ($rep_tester: expr, name = $name:expr, len=$len:expr, before = {$($before:tt)*}, block = {$($block:tt)*}, check = {$check:expr}, after_run={$($after:tt)*}) => {{
        let len = $len;
        $rep_tester.init($name, len as u64, 3.0);

        while $rep_tester.should_continue() {
            $($before)*
            $rep_tester.start_run();
            $($block)*
            $rep_tester.end_run();

            if !$check {
                $rep_tester.error("didn't pass validity check");
            }

            $rep_tester.print();

            $($after)*
        }

        $rep_tester.print();
        $rep_tester.clear();
    }};

    ($rep_tester: expr, name = $name:expr, len=$len:expr, before = {$($before:tt)*}, block = {$($block:tt)*}, check = {$check:expr}$(,)?) => {
        rep_run!(
            $rep_tester, name = $name, len=$len, before = {$($before)*}, block = {$($block)*}, check = {$check}, after_run = {}
        );
    }
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
    }
}
