use std::arch::asm;
use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        pub fn clocks_now() -> u64 {
            let mut clocks: u64;
            unsafe {
                asm!("isb", "mrs {time}, cntvct_el0", time = out(reg) clocks);
            }

            clocks
        }
    } else {
        pub fn clocks_now() -> u64 {
            unsafe { core::arch::x86::_rdtsc() }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "aarch64", not(feature="measure_timer_frequency")))] {
        pub fn detect_clock_frequency(max_estimation_time: Duration) -> u64 {
            let mut clocks: u64;
            unsafe {
                asm!("mrs {time}, cntfrq_el0", time = out(reg) clocks);
            }

            clocks
        }
    } else {
        pub fn detect_clock_frequency(max_estimation_time: Duration) -> u64 {
            use std::time::Instant;

            let clocks_at_start = clocks_now();
            let start_instant = Instant::now();
            loop {
                if start_instant.elapsed() >= max_estimation_time {
                    break;
                }
            }
            let clocks_at_end = clocks_now();

            let clocks_delta = clocks_at_end - clocks_at_start;
            let amount = (clocks_delta as f64) / max_estimation_time.as_secs_f64();

            amount as u64
        }
    }
}
