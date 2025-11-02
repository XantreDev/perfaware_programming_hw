use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use crate::time::TimeMeasurer;

struct Point {
    name: String,
    clocks: u64,
}

struct Profiler {
    measurer: TimeMeasurer,
    labels: Vec<Point>,
    root_start: u64,
}

impl Profiler {
    fn new() -> Option<Profiler> {
        let measurer = TimeMeasurer::init();

        measurer.map(|measurer| {
            let root_start = measurer.clocks_now();

            Profiler {
                labels: Vec::with_capacity(255),
                measurer,
                root_start: root_start,
            }
        })
    }
}
static mut PROFILER: Option<Profiler> = None;

pub fn start_profile() {
    unsafe {
        assert!(matches!(PROFILER, None));
        let _profiler = Profiler::new();
        assert!(matches!(_profiler, Some(_)));
        PROFILER = _profiler;
    }
}

#[allow(static_mut_refs)]
pub fn profile_with_label<F, R>(label: &str, block: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let reference = PROFILER.as_mut();
        if reference.is_none() {
            return block();
        }

        let reference = reference.unwrap();

        let start = reference.measurer.clocks_now();
        let result = block();
        let delta = reference.measurer.clocks_now() - start;

        reference.labels.push(Point {
            name: label.into(),
            clocks: delta,
        });

        result
    }
}

#[allow(static_mut_refs)]
pub fn finish_end_print_root_profile() -> Result<(), String> {
    unsafe {
        let profiler = PROFILER.as_mut().unwrap();

        let end = profiler.measurer.clocks_now();
        let clock_frequency = profiler
            .measurer
            .detect_clock_frequency(Duration::from_millis(50));

        let total_execution_time_clocks = end - profiler.root_start;
        let total_time =
            ((total_execution_time_clocks as f64) / (clock_frequency as f64)) * 1_000.0;

        let mut map: BTreeMap<String, (u64, u32)> = BTreeMap::new();

        for label in &profiler.labels {
            let (value, occurance) = map.get(&label.name).unwrap_or(&(0, 0));
            map.insert(label.name.to_owned(), (value + label.clocks, occurance + 1));
        }
        let mut labels_times: Vec<String> = Vec::with_capacity(map.len());
        for (label, (clocks, occurance)) in map {
            let percentage = ((clocks as f64) / (total_execution_time_clocks as f64)) * 100.0;

            labels_times.push(format!(
                "- {}[{}]={} ({:.2}%)",
                label, occurance, clocks, percentage
            ));
        }

        println!(
            "Execution time: {:.2}ms; CPU Frequency ~{}Hz\n{}",
            total_time,
            total_execution_time_clocks,
            labels_times.join("\n")
        );
        PROFILER = None;

        Ok(())
    }
}
