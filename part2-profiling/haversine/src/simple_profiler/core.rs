use std::time::Duration;

use crate::{pretty_print, time::TimeMeasurer};

#[derive(Clone, Copy)]
struct Anchor {
    total: u64,
    nested: u64,
    occurance: u32,
    last_switch: u64,
    dep0_switch: u64,
    depth: u32,
}
impl Anchor {
    fn empty() -> Anchor {
        Anchor {
            total: 0,
            occurance: 0,
            nested: 0,
            last_switch: 0,
            dep0_switch: 0,
            depth: 0,
        }
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

            let mut prof = Profiler {
                anchors: Box::new([Anchor::empty(); 4096]),
                measurer,
                root_start: now,
            };

            prof.anchors[0].last_switch = now;

            prof
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
    parent: u32,
}

#[allow(static_mut_refs)]
impl Drop for Mark {
    #[inline(always)]
    fn drop(&mut self) {
        let profiler = unsafe {
            PROFILER
                .as_mut()
                .expect("invariant, anchor must be created with profiler init")
        };

        let now = profiler.measurer.clocks_now();

        let cur_anchor = &mut profiler.anchors[self.idx as usize];

        let delta = now - cur_anchor.last_switch;

        cur_anchor.total += delta;
        cur_anchor.depth -= 1;
        cur_anchor.last_switch = now;

        if cur_anchor.depth == 0 {
            cur_anchor.nested += now - cur_anchor.dep0_switch;
        }

        let parent_anchor = &mut profiler.anchors[self.parent as usize];

        unsafe { CUR_SCOPE = self.parent }

        parent_anchor.last_switch = now;
    }
}

impl Mark {
    #[inline(always)]
    fn new(idx: u32, profiler: &mut Profiler) -> Mark {
        let parent = unsafe { CUR_SCOPE };
        let now = profiler.measurer.clocks_now();
        let prev_anchor = &mut profiler.anchors[parent as usize];
        prev_anchor.total += now - prev_anchor.last_switch;

        let next_label = &mut profiler.anchors[idx as usize];

        if next_label.depth == 0 {
            next_label.dep0_switch = now;
        }
        next_label.last_switch = now;
        next_label.occurance += 1;
        next_label.depth += 1;

        unsafe { CUR_SCOPE = idx }
        Mark { idx, parent }
    }
}

#[allow(static_mut_refs)]
pub fn mark_scope(idx: u32) -> Mark {
    let reference = unsafe {
        PROFILER
            .as_mut()
            .expect("mark scope must be used regions wrapped with root profiler")
    };

    Mark::new(idx, reference)
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
            let total = label.total;
            let nested = label.nested;

            let percentage = ((total as f64) / (total_execution_time_clocks as f64)) * 100.0;
            if total != nested {
                let percentage_nested =
                    (((nested) as f64) / (total_execution_time_clocks as f64)) * 100.0;
                labels_times.push(format!(
                    "- {}[{}]={} ({:.2}%, {:.2}% w/children)",
                    label_str, label.occurance, label.total, percentage, percentage_nested
                ));
            } else {
                labels_times.push(format!(
                    "- {}[{}]={} ({:.2}%)",
                    label_str, label.occurance, label.total, percentage,
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
