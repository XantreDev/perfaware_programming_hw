use std::{
    io::{self, Stdout, Write, stdout},
    time::Duration,
    u64,
};

use crate::{pretty_print_u64, time::TimeMeasurer};

#[derive(Debug)]
enum Status {
    Uninit,
    Testing,
    Errored,
    Finished,
}
struct RepRun {
    name: &'static str,
    bytes: u64,
    max: u64,
    total: u64,
    min: u64,
    avg: f64,
}

static EMPTY: &'static str = "";
impl RepRun {
    #[inline(always)]
    fn empty() -> RepRun {
        RepRun {
            name: EMPTY,
            bytes: 0,
            max: 0,
            total: 0,
            avg: 0.0,
            min: u64::MAX,
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.name = EMPTY;
        self.max = 0;
        self.avg = 0.0;
        self.min = u64::MAX;
        self.bytes = 0;
        self.total = 0;
    }
}
pub struct RepTester {
    status: Status,
    error_message: Option<&'static str>,
    measurer: TimeMeasurer,

    is_running: bool,
    tries: u64,
    try_before: u64,
    timeout: u64,
    timer_frequency: u64,
    start_time: u64,
    counter: u32,

    run: RepRun,
}

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
            start_time: RepTester::INIT,
            tries: RepTester::INIT,
            try_before: RepTester::INIT,
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

    pub fn init(&mut self, name: &'static str, bytes: u64, timeout_sec: f64) {
        let freq = self
            .measurer
            .detect_clock_frequency(Duration::from_millis(100));
        match self.status {
            Status::Uninit => {
                self.run.name = name;
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
                self.start_time = self.measurer.clocks_now();
            }
            _ => {
                self.status = Status::Errored;
                self.error_message = Some("Invalid start_run");
            }
        }
    }

    pub fn end_run(&mut self) {
        let now = self.measurer.clocks_now();
        let elapsed = now - self.start_time;

        match self.status {
            Status::Testing if !self.is_running => {
                self.error("Invalid end_run command");
            }
            Status::Testing if now <= self.start_time => {
                self.error("Time travel is forbidden outside of Hogwarts");
            }
            Status::Testing => {
                self.is_running = false;
                let total = self.run.total + 1;
                self.run.total = total;

                self.run.avg =
                    (self.run.avg * (total - 1) as f64 + (elapsed as f64)) / total as f64;
                let is_min = elapsed < self.run.min;
                if is_min {
                    self.run.min = elapsed;
                    self.try_before = now + self.timeout;
                }
                self.run.max = self.run.max.max(elapsed);
            }
            _ => {
                self.error("Invalid end_run");
            }
        }
    }

    pub fn print(&mut self) {
        match self.status {
            Status::Finished => {
                let mut out = stdout();

                if self.counter == 0 {
                    print_header(&mut out, self.run.name).unwrap();
                    self.counter = 1;
                } else {
                    out.write("\x1b[1A\x1b[2K".as_bytes()).unwrap();
                }

                write!(
                    out,
                    "The best run: {}\nThe worst run: {}\nAverage: {}\n\n",
                    performance_measurement(self.run.min, self.timer_frequency, self.run.bytes),
                    performance_measurement(self.run.max, self.timer_frequency, self.run.bytes),
                    performance_measurement(
                        self.run.avg as u64,
                        self.timer_frequency,
                        self.run.bytes
                    )
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
                if self.counter == 1 {
                    print_header(&mut out, self.run.name).unwrap();
                } else {
                    out.write("\x1b[1A\x1b[2K".as_bytes()).unwrap();
                }

                writeln!(
                    out,
                    "The best run: {}",
                    performance_measurement(self.run.min, self.timer_frequency, self.run.bytes),
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
        self.start_time = RepTester::INIT;
        self.tries = RepTester::INIT;
        self.try_before = RepTester::INIT;
    }
}

fn print_header(out: &mut Stdout, name: &'static str) -> io::Result<()> {
    writeln!(out, "--- {} ---", name)
}
fn performance_measurement(clocks: u64, timer_frequency: u64, bytes: u64) -> String {
    if bytes == 0 || clocks == 0 {
        return String::new();
    }

    let time = clocks as f64 / timer_frequency as f64;

    let throughput = (bytes as f64 / time) / (1024.0 * 1024.0);
    format!(
        "{}({:.2} ms) {:.3} mb/s",
        pretty_print_u64(clocks),
        time * 1000.0,
        throughput
    )
}
