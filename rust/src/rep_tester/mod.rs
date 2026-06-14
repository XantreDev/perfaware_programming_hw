use std::{
    default,
    io::{self, Stdout, Write, stdout},
    time::Duration,
    u64,
};

use crate::{pretty_print_with_options, time::TimeMeasurer};

#[derive(Debug)]
enum Status {
    Uninit,
    Testing,
    Errored,
    Finished,
}

#[repr(usize)]
enum VectorItem {
    Clocks = 1,
    PageFaults,
    __CountIdentsLast,
}
impl VectorItem {
    #[inline(always)]
    fn value(self) -> usize {
        self as usize
    }
}
const VEC_SIZE: usize = VectorItem::__CountIdentsLast as usize;

type RunVector = [u64; VEC_SIZE];
type RunVectorF64 = [f64; VEC_SIZE];

struct RepRun {
    name: Option<String>,
    bytes: u64,
    runs: u64,
    start: RunVector,
    min: RunVector,
    max: RunVector,
    avg: RunVectorF64,
}
impl RepRun {
    const MIN_DEFAULT: u64 = u64::MAX;
    const ZERO: u64 = 0;
    const AVG_DEFAULT: f64 = 0.0;
}

#[cfg(target_family = "unix")]
pub fn page_faults() -> u64 {
    use std::mem::MaybeUninit;

    use libc;
    let mut usage = MaybeUninit::<libc::rusage>::uninit();

    unsafe {
        use libc::getrusage;

        let result = getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr());
        if result != 0 {
            panic!("cannot get page fault - getrusage returned {}", result);
        }

        usage.assume_init_ref().ru_minflt as u64 + usage.assume_init_ref().ru_majflt as u64
    }
}

impl RepRun {
    #[inline(always)]
    fn empty() -> RepRun {
        RepRun {
            name: None,
            bytes: 0,
            runs: 0,

            start: [RepRun::ZERO; VEC_SIZE],

            avg: [RepRun::AVG_DEFAULT; VEC_SIZE],
            max: [RepRun::ZERO; VEC_SIZE],
            min: [RepRun::MIN_DEFAULT; VEC_SIZE],
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.name = None;
        self.bytes = 0;
        self.runs = 0;

        self.start.fill(RepRun::ZERO);

        self.avg.fill(RepRun::AVG_DEFAULT);
        self.max.fill(RepRun::ZERO);
        self.min.fill(RepRun::MIN_DEFAULT);
    }
}
pub struct RepTester {
    status: Status,
    error_message: Option<&'static str>,
    measurer: TimeMeasurer,

    is_running: bool,
    try_before: u64,
    timeout: u64,
    timer_frequency: u64,
    counter: u32,

    run: RepRun,
    pub print: bool,
}

pub enum MeasurementKind {
    Best,
    Worst,
    Avg,
}

const EMPTY: &'static str = "";

impl RepTester {
    const INIT: u64 = 0;

    pub fn new() -> Option<RepTester> {
        TimeMeasurer::init().map(|measurer| RepTester {
            status: Status::Uninit,
            error_message: None,
            measurer,
            is_running: false,
            counter: 0,
            run: RepRun::empty(),
            timeout: RepTester::INIT,
            timer_frequency: RepTester::INIT,
            try_before: RepTester::INIT,
            print: true,
        })
    }
    #[inline]
    pub fn error(&mut self, err: &'static str) {
        match self.status {
            Status::Errored => {}
            _ => {
                self.status = Status::Errored;
                self.error_message = Some(err);
            }
        }
    }

    pub fn init(&mut self, name: &str, bytes: u64, timeout_sec: f64) {
        let freq = self
            .measurer
            .detect_clock_frequency(Duration::from_millis(100));
        match self.status {
            Status::Uninit => {
                self.run.name = Some(name.to_owned());
                self.run.bytes = bytes;
                self.status = Status::Testing;
                self.error_message = None;
                self.timeout = (freq as f64 * timeout_sec) as u64;
                self.timer_frequency = freq;

                self.try_before = self.measurer.clocks_now() + self.timeout;
            }
            _ => {
                self.status = Status::Errored;
                self.error_message = Some("Failed to re-init uncleared RepTester");
            }
        }
    }
    pub fn should_continue(&mut self) -> bool {
        match self.status {
            Status::Testing if self.measurer.clocks_now() < self.try_before => true,
            Status::Testing => {
                self.status = Status::Finished;
                false
            }
            _ => false,
        }
    }

    pub fn start_run(&mut self) {
        match self.status {
            Status::Testing if self.is_running => {
                self.status = Status::Errored;
                self.error_message = Some("Double start have occured");
            }
            Status::Testing => {
                self.is_running = true;
                self.run.start[VectorItem::Clocks.value()] = self.measurer.clocks_now();
                self.run.start[VectorItem::PageFaults.value()] = page_faults();
            }
            _ => {
                self.status = Status::Errored;
                self.error_message = Some("Invalid start_run");
            }
        }
    }

    pub fn end_run(&mut self) {
        let now = self.measurer.clocks_now();
        let page_faults = page_faults();

        match self.status {
            Status::Testing if !self.is_running => {
                self.error("Invalid end_run command");
            }
            Status::Testing if now <= self.run.start[VectorItem::Clocks.value()] => {
                self.error("Time travel is forbidden outside of Hogwarts");
            }
            Status::Testing => {
                self.is_running = false;
                let total = self.run.runs + 1;
                self.run.runs = total;

                let current_vec: RunVector = [
                    0,
                    now - self.run.start[VectorItem::Clocks.value()],
                    page_faults - self.run.start[VectorItem::PageFaults.value()],
                ];

                for i in 0..current_vec.len() {
                    self.run.avg[i] = (self.run.avg[i] * (total - 1) as f64
                        + (current_vec[i] as f64))
                        / total as f64;
                }

                if current_vec[VectorItem::Clocks.value()]
                    < self.run.min[VectorItem::Clocks.value()]
                {
                    self.run.min = current_vec;
                    self.try_before = now + self.timeout;
                }
                if current_vec[VectorItem::Clocks.value()]
                    > self.run.max[VectorItem::Clocks.value()]
                {
                    self.run.max = current_vec;
                }
            }
            _ => {
                self.error("Invalid end_run");
            }
        }
    }

    pub fn measurement(&self, kind: MeasurementKind) -> PerformanceMeasurement {
        match kind {
            MeasurementKind::Avg => {
                PerformanceMeasurement::new(self.run.avg, self.timer_frequency, self.run.bytes)
            }
            MeasurementKind::Best => PerformanceMeasurement::new(
                to_run_vector_f64(&self.run.min),
                self.timer_frequency,
                self.run.bytes,
            ),
            MeasurementKind::Worst => PerformanceMeasurement::new(
                to_run_vector_f64(&self.run.max),
                self.timer_frequency,
                self.run.bytes,
            ),
        }
    }

    pub fn print(&mut self) {
        match self.status {
            Status::Finished => {
                let mut out = stdout();
                let name = self.run.name.as_ref().expect("must have a name");

                if self.counter == 0 {
                    print_header(&mut out, name).unwrap();
                    self.counter = 1;
                } else {
                    out.write("\x1b[1A\x1b[2K".as_bytes()).unwrap();
                }

                write!(
                    out,
                    "The best run: {}\nThe worst run: {}\nAverage: {}\n\n",
                    self.measurement(MeasurementKind::Best).to_string(),
                    self.measurement(MeasurementKind::Worst).to_string(),
                    self.measurement(MeasurementKind::Avg).to_string(),
                )
                .unwrap();
                out.flush().unwrap();
            }
            Status::Errored => {
                println!(
                    "Tester errored with {}",
                    self.error_message.unwrap_or(EMPTY)
                );
            }
            Status::Testing if self.counter % 10 != 0 => {
                self.counter += 1;
            }
            Status::Testing => {
                self.counter += 1;
                let mut out = stdout();
                let name = self.run.name.as_ref().expect("must have a name");
                if self.counter == 1 {
                    print_header(&mut out, name).unwrap();
                } else {
                    out.write("\x1b[1A\x1b[2K".as_bytes()).unwrap();
                }

                writeln!(
                    out,
                    "The best run: {}",
                    self.measurement(MeasurementKind::Best).to_string()
                )
                .unwrap();
                out.flush().unwrap();
            }
            Status::Uninit => {
                println!("Tester in {:?} state", self.status);
            }
        }
    }
    pub fn clear(&mut self) {
        self.status = Status::Uninit;
        self.error_message = None;

        self.run.clear();

        self.is_running = false;
        self.counter = 0;
        self.timeout = RepTester::INIT;
        self.timer_frequency = RepTester::INIT;
        self.try_before = RepTester::INIT;
    }
}

fn print_header(out: &mut Stdout, name: &str) -> io::Result<()> {
    writeln!(out, "--- {} ---", name)
}

fn to_run_vector_f64(vector: &RunVector) -> RunVectorF64 {
    vector.map(|it| it as f64)
}

#[derive(Default, Clone, Copy)]
pub struct PerformanceMeasurement {
    pub bytes: u64,
    pub time: f64,
    pub faults: f64,
    pub clocks: f64,
}

impl PerformanceMeasurement {
    #[inline]
    pub fn throughput_mb(&self) -> f64 {
        if self.time == 0.0 {
            return 0.0;
        }

        return (self.bytes as f64 / self.time) / (1024.0 * 1024.0);
    }
    pub fn to_string(&self) -> String {
        let clocks = self.clocks;
        let bytes = self.bytes;

        if bytes == 0 || clocks == 0.0 && self.time == 0.0 {
            return String::new();
        }

        let faults = self.faults;
        let page_faults = if faults > 0.0 {
            let mut pf_per_byte = bytes as f64 / faults;
            let unit = if pf_per_byte > 1024.0 * 1024.0 {
                pf_per_byte /= 1024.0 * 1024.0;
                "m"
            } else if pf_per_byte > 1024.0 {
                pf_per_byte /= 1024.0;
                "k"
            } else {
                "b"
            };

            format!(
                "; PF={} ({}{}/fault)",
                pretty_print_with_options(faults, 3),
                pretty_print_with_options(pf_per_byte, 3),
                unit
            )
        } else {
            String::new()
        };
        let throughput = (bytes as f64 / self.time) / (1024.0 * 1024.0);

        format!(
            "{}({:.2} ms) {:.3} mb/s{}",
            pretty_print_with_options(clocks, 3),
            self.time * 1000.0,
            throughput,
            page_faults
        )
    }

    fn new(counts: RunVectorF64, timer_frequency: u64, bytes: u64) -> PerformanceMeasurement {
        let clocks = counts[VectorItem::Clocks.value()];
        if bytes == 0 || clocks == 0.0 {
            return PerformanceMeasurement::nil();
        }

        let time = clocks as f64 / timer_frequency as f64;
        let page_faults = counts[VectorItem::PageFaults.value()];

        return PerformanceMeasurement {
            bytes,
            time,
            faults: page_faults,
            clocks,
        };
    }
    fn nil() -> PerformanceMeasurement {
        PerformanceMeasurement {
            bytes: 0,
            time: 0.0,
            faults: 0.0,
            clocks: 0.0,
        }
    }
}

#[macro_export]
macro_rules! rep_run {
    ($rep_tester: expr, name = $name:expr, len=$len:expr, before = {$($before:tt)*}, block = {$($block:tt)*}, check = {$check:expr}, after_run={$($after:tt)*}) => {{
        let len = $len;
        #[cfg(feature = "precise_rep_test")]
        const TIMEOUT: f64 = 10.0;
        #[cfg(not(feature = "precise_rep_test"))]
        const TIMEOUT: f64 = 3.0;
        $rep_tester.clear();
        $rep_tester.init($name, len as u64, TIMEOUT);

        while $rep_tester.should_continue() {
            $($before)*
            $rep_tester.start_run();
            $($block)*
            $rep_tester.end_run();

            if !$check {
                $rep_tester.error("didn't pass validity check");
            }

            if $rep_tester.print {
                $rep_tester.print();
            }

            $($after)*
        }

        if $rep_tester.print {
            $rep_tester.print();
        }
    }};

    ($rep_tester: expr, name = $name:expr, len=$len:expr, block = {$($block:tt)*} $(,)?) => {
        rep_run!(
            $rep_tester, name = $name, len=$len, before = {}, block = {$($block)*}, check = {true}, after_run = {}
        );
    };

    ($rep_tester: expr, name = $name:expr, len=$len:expr, before = {$($before:tt)*}, block = {$($block:tt)*} $(,)?) => {
        rep_run!(
            $rep_tester, name = $name, len=$len, before = {$($before)*}, block = {$($block)*}, check = {true}, after_run = {}
        );
    };

    ($rep_tester: expr, name = $name:expr, len=$len:expr, before = {$($before:tt)*}, block = {$($block:tt)*}, check = {$check:expr}$(,)?) => {
        rep_run!(
            $rep_tester, name = $name, len=$len, before = {$($before)*}, block = {$($block)*}, check = {$check}, after_run = {}
        );
    }
}
