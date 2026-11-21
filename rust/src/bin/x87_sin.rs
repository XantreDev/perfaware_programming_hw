use std::arch::asm;
use std::hint::black_box;

use haversine_generator::{rep_run, setup_rep_test_single_core};

#[inline(never)]
unsafe fn benchmark_fsin(n: u64) -> f64 {
    assert!(n > 0);
    let mut result = 0.0f64;

    unsafe {
        // llm asm, I am not sure bout it
        asm!(
            "fld1",
            "2:",
            "fsin",
            "dec {n}",
            "jnz 2b",
            "fstp qword ptr [{out}]",

            n = inout(reg) n => _,
            out = in(reg) &mut result,

            out("st(0)") _,
            out("st(1)") _,
            out("st(2)") _,
            out("st(3)") _,
            out("st(4)") _,
            out("st(5)") _,
            out("st(6)") _,
            out("st(7)") _,
            options(nostack),
        );
    }

    result
}

fn main() {
    let n = 1_000_000_u64;
    let mut rep_tester = setup_rep_test_single_core().unwrap();

    rep_run!(
        rep_tester,
        name = "x87_sin",
        len = n,
        block = {
            let r = unsafe { benchmark_fsin(black_box(n)) };
            black_box(r);
        }
    );
    rep_run!(
        rep_tester,
        name = "cmath_sin",
        len = n,
        block = {
            for _ in 0..n {
                let r = black_box(0.5 as f64).sin();
                black_box(r);
            }
        }
    );
}
