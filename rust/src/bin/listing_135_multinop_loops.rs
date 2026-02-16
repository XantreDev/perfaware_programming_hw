use std::{arch::asm, env::args};

use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

fn main() {
    core_affinity::set_single_core().unwrap();
    let mut rep_tester = RepTester::new().unwrap();
    let iterations = {
        let mut args = args();

        args.nth(1)
            .map(|it| it.parse::<u32>().ok())
            .flatten()
            .expect("[iterations_count]")
    };

    loop {
        rep_run!(
            rep_tester,
            name = "nop3Bx1",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x0F, 0x1F, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );
        rep_run!(
            rep_tester,
            name = "nop1Bx3",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x90
                            .byte 0x90
                            .byte 0x90
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );

        rep_run!(
            rep_tester,
            name = "nop3Bx3",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                    r#"
                        2:
                            .byte 0x0F, 0x1F, 0x00
                            .byte 0x0F, 0x1F, 0x00
                            .byte 0x0F, 0x1F, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );
        rep_run!(
            rep_tester,
            name = "nop1Bx9",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x90
                            .byte 0x90
                            .byte 0x90

                            .byte 0x90
                            .byte 0x90
                            .byte 0x90

                            .byte 0x90
                            .byte 0x90
                            .byte 0x90

                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );
        rep_run!(
            rep_tester,
            name = "nop9Bx1",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );

        rep_run!(
            rep_tester,
            name = "nop9Bx3",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );

        rep_run!(
            rep_tester,
            name = "nop9Bx10",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );

        rep_run!(
            rep_tester,
            name = "nop9Bx30",
            len = iterations,
            block = {
                unsafe {
                    asm!(
                        r#"
                        2:
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            .byte 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
                            inc {in:r}
                            cmp {in:r}, {target:r}
                            jb 2b
                        "#,
                        in = in(reg) 0,
                        target = in(reg) iterations,
                        options(nostack)
                    );
                }
            }
        );
    }
}
