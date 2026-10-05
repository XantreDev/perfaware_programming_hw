use std::{env, fs::File, io::Read, ops::Not, path::Path, process::exit};

use haversine_generator::{rep_run, setup_rep_test_single_core, write::RawAlloc};

unsafe fn rep_movsb(dst: *mut u8, src: *const u8, n: usize) {
    unsafe {
        core::arch::asm!(
            "rep movsb",
            inout("rdi") dst => _,
            inout("rsi") src => _,
            inout("rcx") n => _,
            options(nostack)
        );
    }
}

fn main() {
    let mut rep_tester = setup_rep_test_single_core().unwrap();
    let mut args = env::args();
    if args.len() < 2 {
        println!("[inputFilePath] is required");
        exit(1);
    }
    let file_path = args.nth(1).unwrap();
    let path = Path::new(&file_path);
    let file = File::open(&path).unwrap().metadata().unwrap();
    const PAGE: usize = 4096;
    let len = (file.len() as usize + PAGE) & (PAGE - 1).not();
    let src_alloc = RawAlloc::new(len as usize);
    let src = src_alloc.as_u8_slice_mut();
    for i in 0..src.len() {
        src[i] = ((i * 10) & 255) as u8;
    }

    const MAX_ALIGN: usize = 128;
    for i in (0..MAX_ALIGN).step_by(16) {
        let name = format!("offset {}", i);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            before = {
                let dst_buf = RawAlloc::new(len as usize + MAX_ALIGN);
                let dst = &mut dst_buf.as_u8_slice_mut()[i..i + (len as usize)];
            },
            block = {
                for page in 0..(len as usize / PAGE) {
                    unsafe {
                        rep_movsb(
                            dst.as_mut_ptr().add(PAGE * page),
                            src.as_ptr().add(PAGE * page),
                            4096,
                        );
                    }
                }
            },
        );
    }
}
