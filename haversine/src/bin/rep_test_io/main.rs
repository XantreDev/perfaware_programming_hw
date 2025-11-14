use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    process::exit,
};

use haversine_generator::rep_tester::RepTester;

struct TestFn<'a> {
    name: &'static str,
    args: &'a String,
    exepcted_size: u64,
    block: &'a dyn Fn(&String) -> String,
}

fn run_with_reps(rep_tester: &mut RepTester, test: &TestFn) {
    rep_tester.init(test.name, test.exepcted_size, 3.0);
    while rep_tester.should_continue() {
        rep_tester.start_run();
        let block = test.block;
        let json = block(test.args);

        rep_tester.end_run();
        if json.len() != test.exepcted_size as usize {
            rep_tester.error("invalid size");
        }
        rep_tester.print();
    }

    rep_tester.print();
    rep_tester.clear();
}

fn fs_read_to_string(arg: &String) -> String {
    fs::read_to_string(&arg).unwrap()
}
fn file_read_to_string(arg: &String) -> String {
    let mut file = fs::File::open(arg).unwrap();
    let mut str = String::new();
    file.read_to_string(&mut str).unwrap();

    str
}

fn file_read_to_string_with_capacity(arg: &String) -> String {
    let mut file = fs::File::open(arg).unwrap();
    let mut str = String::new();
    file.read_to_string(&mut str).unwrap();

    str
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
    let testers = vec![
        TestFn {
            name: "File::read_to_string",
            args: &_arg,
            exepcted_size: meta.len(),
            block: &file_read_to_string,
        },
        TestFn {
            name: "fs::read_to_string",
            args: &_arg,
            exepcted_size: meta.len(),
            block: &fs_read_to_string,
        },
        TestFn {
            name: "File::read_to_string + with_capacity",
            args: &_arg,
            exepcted_size: meta.len(),
            block: &file_read_to_string_with_capacity,
        },
    ];

    let mut i = 0;
    loop {
        run_with_reps(&mut rep_tester, &testers[i]);

        i += 1;
        if i == testers.len() {
            i = 0;
        }
    }
}
