use std::{arch::asm, env::args, hint::unreachable_unchecked, io};

use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

fn run_loop(slice: &mut [u8]) {
    unsafe {
        asm!(
            r#"
                xor {val:l}, {val:l}
                xor {idx}, {idx}
            2:
                mov {val:l}, [{ptr} + {idx}]
                test {val:l}, 1
                jne 3f
                # nop 3B
                .byte 0x0F, 0x1F, 0x00
            3:
                inc {idx}
                cmp {idx}, {amount}
                jbe 2b
            "#,
            idx=out(reg) _,
            val=out(reg) _,
            ptr=in(reg) slice.as_mut_ptr(),
            amount=in(reg) (slice.len() as u64),
            options(nostack)
        );
    };
}
#[derive(Clone, Copy)]
enum BranchPattern {
    Never,
    Always,
    Every(u8),
    OSRandom,
}

impl BranchPattern {
    fn name(self) -> Result<&'static str, ()> {
        let name = match self {
            BranchPattern::Always => "BranchAlways",
            BranchPattern::Never => "BranchNever",
            BranchPattern::Every(0) => "BranchEvery0",
            BranchPattern::Every(1) => "BranchEvery1",
            BranchPattern::Every(2) => "BranchEvery2",
            BranchPattern::Every(3) => "BranchEvery3",
            BranchPattern::Every(4) => "BranchEvery4",
            BranchPattern::Every(8) => "BranchEvery8",
            BranchPattern::Every(_) => return Err(()),
            BranchPattern::OSRandom => "BranchOSRandom",
        };

        Ok(name)
    }
}

fn fill_bytes(bytes: &mut [u8], pattern: BranchPattern) -> Result<(), io::Error> {
    if matches!(pattern, BranchPattern::OSRandom) {
        let mut filled = 0;

        static SIZE: usize = 128;
        let mut entropy = [0u8; SIZE];

        loop {
            unsafe {
                let res =
                    libc::getrandom(entropy.as_mut_ptr() as *mut libc::c_void, entropy.len(), 0);
                if res == -1 {
                    return Err(io::Error::last_os_error());
                }
            };
            let entropy_bits = SIZE * 8;
            let bits_to_fill = bytes.len() - filled;

            for i in 0..(entropy_bits.min(bits_to_fill)) {
                let bit = entropy[i >> 3].unbounded_shr((i & 7) as u32);
                bytes[filled + i] = bit;
            }

            filled += entropy_bits;
            if filled >= bytes.len() {
                return Ok(());
            }
        }
    }

    for (i, value) in bytes.iter_mut().enumerate() {
        match pattern {
            BranchPattern::Always => {
                *value = 1;
            }
            BranchPattern::Never => {
                *value = 0;
            }
            BranchPattern::Every(pattern) => {
                *value = ((i % (pattern as usize)) == 0).into();
            }
            BranchPattern::OSRandom => unsafe {
                unreachable_unchecked();
            },
        }
    }

    Ok(())
}

fn main() {
    let bytes = {
        let mut args = args();
        args.nth(1)
            .expect("[iteration_bytes:u32]")
            .parse::<u32>()
            .expect("[iteration_bytes:u32]") as usize
    };

    core_affinity::set_single_core().unwrap();

    let mut arr = vec![0u8; bytes];
    let mut rep_tester = RepTester::new().unwrap();

    loop {
        let patterns = [
            BranchPattern::Never,
            BranchPattern::Always,
            BranchPattern::Every(2),
            BranchPattern::Every(3),
            BranchPattern::Every(4),
            BranchPattern::Every(8),
            BranchPattern::OSRandom,
        ];
        for pat in patterns {
            fill_bytes(&mut arr, pat.clone()).unwrap();

            rep_run!(
                rep_tester,
                name = pat.clone().name().unwrap(),
                len = bytes,
                block = {
                    run_loop(&mut arr);
                }
            );
        }
    }
}
