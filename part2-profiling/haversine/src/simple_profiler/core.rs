use std::time::Duration;

use crate::{pretty_print, time::TimeMeasurer};

#[derive(Clone, Copy)]
struct Anchor {
    inclusive: u64,
    exclusive: u64,
    occurance: u32,
}

impl Anchor {
    fn empty() -> Anchor {
        Anchor {
            inclusive: 0,
            occurance: 0,
            exclusive: 0,
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
    fn new(idx: u32) -> Mark {
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
        };

        unsafe { CUR_SCOPE = idx };

        mark
    }
}

#[allow(static_mut_refs)]
pub fn mark_scope(idx: u32) -> Mark {
    Mark::new(idx)
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
        let total_time =
            ((total_execution_time_clocks as f64) / (clock_frequency as f64)) * 1_000.0;

        let mut labels_times: Vec<String> = Vec::with_capacity(labels.len());

        for (idx, label_str) in labels {
            let label = &profiler.anchors[*idx as usize];
            if label.exclusive == 0 && label.inclusive == 0 {
                continue;
            }
            let percentage =
                (((label.exclusive) as f64) / (total_execution_time_clocks as f64)) * 100.0;

            if label.inclusive != label.exclusive {
                let percentage_nested =
                    (((label.inclusive) as f64) / (total_execution_time_clocks as f64)) * 100.0;

                labels_times.push(format!(
                    "- {}[{}]={} ({:.2}%, {:.2}% w/children)",
                    label_str,
                    label.occurance(),
                    pretty_print(label.inclusive as f64),
                    percentage,
                    percentage_nested
                ));
            } else {
                labels_times.push(format!(
                    "- {}[{}]={} ({:.2}%)",
                    label_str,
                    label.occurance(),
                    pretty_print(label.inclusive as f64),
                    percentage,
                ));
            }
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
