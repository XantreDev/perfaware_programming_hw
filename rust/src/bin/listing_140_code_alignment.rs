use std::env::args;

use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

type CodeAlignLoop = unsafe extern "C" fn(u64);
struct CodeAlignmentKind {
    name: &'static str,
    ptr: CodeAlignLoop,
}
impl CodeAlignmentKind {
    fn new(name: &'static str, ptr: CodeAlignLoop) -> CodeAlignmentKind {
        CodeAlignmentKind { name, ptr }
    }
}
// https://www.computerenhance.com/p/code-alignment
pub fn main() {
    let iterations = {
        let mut args = args();

        args.nth(1)
            .expect("[iteration_count:u32]")
            .replace('_', "")
            .parse::<u32>()
            .expect("[iteration_count:u32]") as usize
    };

    core_affinity::set_single_core().unwrap();

    let mut rep_tester = RepTester::new().unwrap();

    loop {
        let alignments = [
            CodeAlignmentKind::new("align_64", asm::alignment::align_64),
            CodeAlignmentKind::new("misalign_1", asm::alignment::misalign_1),
            CodeAlignmentKind::new("misalign_8", asm::alignment::misalign_8),
            CodeAlignmentKind::new("misalign_16", asm::alignment::misalign_16),
            CodeAlignmentKind::new("misalign_32", asm::alignment::misalign_32),
            CodeAlignmentKind::new("misalign_48", asm::alignment::misalign_48),
            CodeAlignmentKind::new("misalign_62", asm::alignment::misalign_62),
            CodeAlignmentKind::new("misalign_63", asm::alignment::misalign_63),
        ];

        for it in alignments {
            rep_run!(
                rep_tester,
                name = it.name,
                len = iterations,
                block = {
                    unsafe {
                        let ptr = it.ptr;
                        ptr(iterations as u64);
                    }
                },
            );
        }
    }
}
