use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
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
            pub fn clocks_now(&mut self) -> u64 {
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
    } else {
        pub struct TimeMeasurer;
        impl TimeMeasurer {
            pub fn init() -> Option<TimeMeasurer> {
                Some(TimeMeasurer {});
            }

            pub fn clocks_now(&mut self) -> u64 {
                unsafe { core::arch::x86::_rdtsc() }
            }
        }
    }
}

impl TimeMeasurer {
    pub fn detect_clock_frequency(&mut self, max_estimation_time: Duration) -> u64 {
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
