use std::{arch::asm, env::args};

use haversine_generator::{
    core_affinity, rep_run,
    rep_tester::{self, RepTester},
    write::RawAlloc,
};

fn main() {
    core_affinity::set_single_core().unwrap();
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
                        2:
                        mov [{ptr} + {c:r}], {c:l}
                        inc {c:r}
                        cmp {c:r}, {bound:r}
                        jb 2b"#,
                        c = in(reg) 0,
                        ptr = in(reg) ptr,
                        bound = in(reg) bytes,
                        options(nostack)
                    );
                }
            },
        );

        rep_run!(
            rep_tester,
            name = "nop (aligned)",
            len = bytes,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                        NOP DWORD ptr [EAX]
                        inc {c:r}
                        cmp {c:r}, {bound:r}
                        jb 2b"#,
                        c = in(reg) 0,
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
                        2:
                        .byte 0x0F, 0x1F, 0x00H
                        inc {c:r}
                        cmp {c:r}, {bound:r}
                        jb 2b"#,
                        c = in(reg) 0,
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
                        2:
                        inc {c:r}
                        cmp {c:r}, {bound:r}
                        jb 2b"#,
                        c = in(reg) 0,
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
                        dec {i:r}
                        cmp {i:r}, 0
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
