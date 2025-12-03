use std::{arch::asm, env::args};

use haversine_generator::{
    rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

fn main() {
    let mut args = args();
    let pages: u32 = args.nth(1).unwrap().parse().unwrap();
    let page = 4 * 1024;
    let max_mem = (1024 * 1024 * 1024 * 2);
    assert!(pages * page < max_mem);

    let bytes = (page * pages) as usize;
    let arr = RawAlloc::new(bytes);

    let mut rep_tester = RepTester::new().unwrap();

    loop {
        let ptr = arr.as_mut_ptr();
        rep_run!(
            rep_tester,
            name = "mov",
            len = bytes,
            block = {
                unsafe {
                    asm!(
                        r#"
                        xor rax, rax
                        2:
                        mov [{ptr} + rax], al
                        inc rax
                        cmp rax, {bound}
                        jb 2b"#,
                        ptr = in(reg) ptr,
                        bound = in(reg) bytes,
                        options(nostack)
                    );
                }
            },
        );

        rep_run!(
            rep_tester,
            name = "nop",
            len = bytes,
            block = {
                unsafe {
                    asm!(
                        r#"
                        xor rax, rax
                        2:
                        NOP DWORD ptr [EAX]
                        inc rax
                        cmp rax, {bound}
                        jb 2b"#,
                        bound = in(reg) bytes,
                        options(nostack)
                    );
                }
            },
        );

        rep_run!(
            rep_tester,
            name = "none",
            len = bytes,
            block = {
                unsafe {
                    asm!(
                        r#"
                        xor rax, rax
                        2:
                        inc rax
                        cmp rax, {bound}
                        jb 2b"#,
                        bound = in(reg) bytes,
                        options(nostack)
                    );
                }
            },
        );

        rep_run!(
            rep_tester,
            name = "dec",
            len = bytes,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                        dec {i}
                        cmp {i}, 0
                        jb 2b
                        "#,
                        i = in(reg) bytes - 1,
                        options(nostack)
                    );
                }
            }
        )
    }
}
