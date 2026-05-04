use aligned::Aligned;
use haversine_generator::{core_affinity, rep_run, rep_tester::RepTester};

fn main() {
    const BUF_BITS: usize = 30;
    const MIN_READ_BITS: usize = 8;
    const BUF_SIZE: usize = 1 << BUF_BITS;

    let mut buf: Box<Aligned<aligned::A64, [u8; BUF_SIZE]>> = Box::new(Aligned([0u8; BUF_SIZE]));

    core_affinity::set_single_core().unwrap();
    let mut tester = RepTester::new().unwrap();

    loop {
        for i in MIN_READ_BITS..=BUF_BITS {
            let name: String;
            if i >= 20 {
                name = format!("cache_{}MB", 1 << (i - 20));
            } else if i >= 10 {
                name = format!("cache_{}kB", 1 << (i - 10));
            } else {
                name = format!("cache_{}B", 1 << i);
            }
            // 4 -> (1 << 4) -> 10000 -> 1111
            let mask = (1 << i) - 1;
            rep_run!(
                tester,
                name = &name,
                len = BUF_SIZE,
                block = {
                    unsafe {
                        asm::cache::test_cache(BUF_SIZE as u64, mask, buf.as_mut_ptr());
                    };
                }
            )
        }
    }
}
