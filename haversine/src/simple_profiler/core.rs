use std::time::Duration;

use crate::{pretty_print, time::TimeMeasurer};

#[derive(Clone, Copy)]
struct Anchor {
    inclusive: u64,
    exclusive: u64,
    occurance: u32,
    processed_bytes: u64,
}

impl Anchor {
    fn empty() -> Anchor {
        Anchor {
            inclusive: 0,
            occurance: 0,
            exclusive: 0,
            processed_bytes: 0,
        }
    }

    fn occurance(&self) -> u32 {
        self.occurance
    }
}

struct Profiler {
    measurer: TimeMeasurer,
    anchors: Box<[Anchor]>,
    root_start: u64,
}

impl Profiler {
    fn new() -> Option<Profiler> {
        let measurer = TimeMeasurer::init();

        measurer.map(|measurer| {
            let now = measurer.clocks_now();

            Profiler {
                anchors: Box::new([Anchor::empty(); 4096]),
                measurer,
                root_start: now,
            }
        })
    }
}
static mut PROFILER: Option<Profiler> = None;
static mut CUR_SCOPE: u32 = 0;

pub fn start_profile() {
    unsafe {
        assert!(matches!(PROFILER, None));
        let _profiler = Profiler::new();
        assert!(matches!(_profiler, Some(_)));
        PROFILER = _profiler;
    }
}

pub struct Mark {
    idx: u32,
    start: u64,
    self_inclusive: u64,
    after_bytes: u64,
    parent: u32,
}

#[allow(static_mut_refs)]
impl Drop for Mark {
    #[inline(always)]
    fn drop(&mut self) {
        self._drop();
    }
}

impl Mark {
    #[inline(always)]
    #[allow(static_mut_refs)]
    fn _drop(&mut self) {
        let prof = unsafe {
            PROFILER
                .as_mut()
                .expect("invariant, anchor must be created with profiler init")
        };

        let elapsed = prof.measurer.clocks_now() - self.start;

        prof.anchors[self.idx as usize].occurance += 1;
        prof.anchors[self.idx as usize].processed_bytes = self.after_bytes;

        prof.anchors[self.idx as usize].inclusive = self.self_inclusive.wrapping_add(elapsed);
        prof.anchors[self.idx as usize].exclusive = prof.anchors[self.idx as usize]
            .exclusive
            .wrapping_add(elapsed);

        prof.anchors[self.parent as usize].exclusive = prof.anchors[self.parent as usize]
            .exclusive
            .wrapping_sub(elapsed);

        unsafe { CUR_SCOPE = self.parent };
    }

    #[inline(always)]
    #[allow(static_mut_refs)]
    fn new(idx: u32, bytes: u64) -> Mark {
        let prof = unsafe {
            PROFILER
                .as_mut()
                .expect("invariant, anchor must be created with profiler init")
        };
        let scope = unsafe { CUR_SCOPE };

        let mark = Mark {
            idx,
            start: prof.measurer.clocks_now(),
            self_inclusive: prof.anchors[idx as usize].inclusive,
            parent: scope,
            after_bytes: prof.anchors[idx as usize].processed_bytes + bytes,
        };

        unsafe { CUR_SCOPE = idx };

        mark
    }
}

#[allow(static_mut_refs)]
pub fn mark_scope(idx: u32, processed_bytes: u64) -> Mark {
    Mark::new(idx, processed_bytes)
}

#[allow(static_mut_refs)]
pub fn finish_end_print_root_profile(labels: &[(u32, &'static str)]) -> Result<(), String> {
    unsafe {
        let profiler = PROFILER.as_mut().unwrap();

        let now = profiler.measurer.clocks_now();
        let clock_frequency = profiler
            .measurer
            .detect_clock_frequency(Duration::from_millis(100));

        let total_execution_time_clocks = (now - profiler.root_start);
        let total_time_s = (total_execution_time_clocks as f64) / (clock_frequency as f64);
        let total_time = total_time_s * 1_000.0;

        let mut labels_times: Vec<String> = Vec::with_capacity(labels.len());

        for (idx, label_str) in labels {
            let label = &profiler.anchors[*idx as usize];
            if label.exclusive == 0 && label.inclusive == 0 {
                continue;
            }
            let percentage =
                (((label.exclusive) as f64) / (total_execution_time_clocks as f64)) * 100.0;

            let empty = String::new();
            let throughput = match label.processed_bytes {
                0 => &empty,
                bytes => {
                    let mbytes = (bytes as f64) / (1024.0 * 1024.0);
                    let gbytes = (bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                    let execution_time_s = label.inclusive as f64 / clock_frequency as f64;
                    let throughput = mbytes / execution_time_s;
                    &format!(" {:.3} GB => {:.2} mb/s", gbytes, throughput)
                }
            };
            let children = if label.inclusive != label.exclusive {
                let percentage_nested =
                    (((label.inclusive) as f64) / (total_execution_time_clocks as f64)) * 100.0;

                &format!(", {:.2}% w/children", percentage_nested)
            } else {
                &empty
            };

            labels_times.push(format!(
                "- {}[{}]={} ({:.2}%{}){}",
                label_str,
                label.occurance(),
                pretty_print(label.inclusive as f64),
                percentage,
                children,
                throughput
            ));
        }

        println!(
            "Execution time: {:.2}ms; CPU Frequency ~{}Hz\n{}",
            total_time,
            pretty_print(clock_frequency as f64),
            labels_times.join("\n")
        );
        PROFILER = None;

        Ok(())
    }
}
