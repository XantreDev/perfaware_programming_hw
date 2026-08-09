use std::{env, fs::File, io::Read, path::Path, process::exit};

use haversine_generator::{rep_run, setup_rep_test, write::RawAlloc};

fn main() {
    let mut rep_tester = setup_rep_test().unwrap();
    let mut args = env::args();
    if args.len() < 2 {
        println!("[inputFilePath] is required");
        exit(1);
    }
    let file_path = args.nth(1).unwrap();
    let path = Path::new(&file_path);
    let file = File::open(&path).unwrap().metadata().unwrap();
    let len = file.len();

    const MAX_ALIGN: usize = 128;
    for i in (0..MAX_ALIGN).step_by(16) {
        let name = format!("offset {}", i);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            before = {
                let buf = RawAlloc::new(len as usize + MAX_ALIGN);
                let arr = &mut buf.as_u8_slice_mut()[i..i + (len as usize)];
                let mut file = File::open(&path).unwrap();
            },
            block = {
                let res = file.read_exact(arr);
            },
            check = { res.is_ok() },
        );
    }
}
