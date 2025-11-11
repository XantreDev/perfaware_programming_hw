use std::{
    alloc::System,
    arch::asm,
    time::{Duration, SystemTime},
};

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "aarch64", feature="timing_mac_os_cycles"))] {
        use libc::{c_int};

        #[repr(C)]
        struct PerfCounters {
            cycles: u64,
            branches: u64,
            missed_branches: u64,
            instructions: u64
        }
        #[repr(C)]
        struct PerfCountersHandle {_private: [u8; 0]}
        unsafe extern "C" {
            fn apple_events_create() -> *const PerfCountersHandle;
            fn apple_events_destroy(h: *const PerfCountersHandle);
            fn apple_events_get(h: *const PerfCountersHandle, out: *const PerfCounters) -> c_int;
        }


        pub struct TimeMeasurer {
            handle: *const PerfCountersHandle,
            counters: Box<PerfCounters>
        }
        impl Drop for TimeMeasurer {
            fn drop(&mut self) {
                unsafe {
                    apple_events_destroy(self.handle);
                }
            }
        }
        impl TimeMeasurer {
            #[inline]
            pub fn init() -> Option<TimeMeasurer> {
                unsafe {
                    let events = apple_events_create();
                    if matches!(events.as_ref(), None) {
                        return None;
                    }
                    let counters = Box::new(PerfCounters {
                        cycles: 0,
                        branches: 0,
                        missed_branches: 0,
                        instructions: 0
                    });

                    return Some(TimeMeasurer {
                        handle: events,
                        counters: counters
                    });
                }
            }
            pub fn clocks_now(&self) -> u64 {
                unsafe {
                    let result = apple_events_get(
                        self.handle,
                        &*self.counters as *const PerfCounters
                    );
                    if result != 0 {
                        return 0;
                    }

                    self.counters.cycles
                }
            }
        }
    } else if #[cfg(feature="timing_os")] {
        use libc::{CLOCK_MONOTONIC, clock_gettime, timespec};
        pub struct TimeMeasurer;

        impl TimeMeasurer {
            pub fn init() -> Option<TimeMeasurer> {
                Some(TimeMeasurer{})
            }
            pub fn clocks_now(&self) -> u64 {
                let mut spec = timespec {tv_nsec: 0, tv_sec: 0};
                #[cfg(target_vendor = "apple")]
                const CLOCK_ID: libc::clockid_t = libc::CLOCK_UPTIME_RAW;
                #[cfg(not(target_vendor = "apple"))]
                const CLOCK_ID: libc::clockid_t = libc::CLOCK_MONOTONIC;
                let code = unsafe {
                     clock_gettime(CLOCK_ID, &mut spec)
                };
                assert!(code == 0);

                const NSEC_PER_SEC: u64 = 1_000_000_000;

                let res = (((spec.tv_sec as u64) & 0xFFFF_FFFF) * NSEC_PER_SEC) + (spec.tv_nsec as u64);

                res
            }
        }
    } else if #[cfg(all(target_arch = "aarch64", feature="timing_low_level"))] {
        pub struct TimeMeasurer;
        impl TimeMeasurer {
            pub fn init() -> Option<TimeMeasurer> {
                Some(TimeMeasurer{})
            }
            pub fn clocks_now(&self) -> u64 {
                unsafe {
                    let mut now: u64 = 0;
                    asm!("mrs {now}, CNTVCT_EL0", now = inout(reg) now);

                    now
                }
            }
        }
    } else if #[cfg(feature="timing_low_level")] {
        pub struct TimeMeasurer;
        impl TimeMeasurer {
            pub fn init() -> Option<TimeMeasurer> {
                Some(TimeMeasurer {});
            }

            pub fn clocks_now(&self) -> u64 {
                unsafe { core::arch::x86::_rdtsc() }
            }
        }
    } else {
        panic!("no matching options")
    }
}

impl TimeMeasurer {
    pub fn detect_clock_frequency(&self, max_estimation_time: Duration) -> u64 {
        use std::time::Instant;

        let clocks_at_start = self.clocks_now();
        let start_instant = Instant::now();
        loop {
            if start_instant.elapsed() >= max_estimation_time {
                break;
            }
        }
        let clocks_at_end = self.clocks_now();

        let clocks_delta = clocks_at_end - clocks_at_start;
        let amount = (clocks_delta as f64) / max_estimation_time.as_secs_f64();

        amount as u64
    }
}
