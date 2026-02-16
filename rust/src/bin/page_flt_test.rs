use std::mem::MaybeUninit;

use haversine_generator::rep_tester::{self, RepTester};

#[cfg(target_family = "unix")]
fn page_faults() -> u64 {
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

fn main() {
    let faults_start = page_faults();
    let mut rep_tester = RepTester::new().unwrap();
    let mut value: i64 = 0;
    let i64_size = i64::BITS as usize / 8;
    let size: usize = 700 * 1024 * 1024 / i64_size;
    let seed = rand::random::<i64>();
    let mut arr: Box<[MaybeUninit<i64>]> = Box::new_uninit_slice(size);
    rep_tester.init("check", (size * i64_size) as u64, 1.0);
    rep_tester.start_run();
    for i in 0..size / 4 {
        let i = i * 4;
        arr[i] = MaybeUninit::new(seed * (i as i64 + 1) * 8);
        arr[i + 1] = MaybeUninit::new(seed * ((i + 1) as i64 + 1) * 8);
        arr[i + 2] = MaybeUninit::new(seed * ((i + 2) as i64 + 1) * 8);
        arr[i + 3] = MaybeUninit::new(seed * ((i + 3) as i64 + 1) * 8);
    }
    rep_tester.end_run();

    value += arr
        .iter()
        .fold(0, |acc, it| acc + unsafe { it.assume_init() });

    rep_tester.print();
    println!("res {}; faults {}", value, page_faults() - faults_start);
}
