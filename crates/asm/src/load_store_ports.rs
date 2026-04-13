use crate::unix_extern;

unix_extern!(
    fn read_1(iterations: u64, addr: *mut u64);
    fn read_2(iterations: u64, addr: *mut u64);
    fn read_3(iterations: u64, addr: *mut u64);
    fn read_4(iterations: u64, addr: *mut u64);

    fn write_1(iterations: u64, a: *mut u64, b: *mut u64, c: *mut u64, d: *mut u64);
    fn write_2(iterations: u64, a: *mut u64, b: *mut u64, c: *mut u64, d: *mut u64);
    fn write_3(iterations: u64, a: *mut u64, b: *mut u64, c: *mut u64, d: *mut u64);
    fn write_4(iterations: u64, a: *mut u64, b: *mut u64, c: *mut u64, d: *mut u64);

    fn read_1x2(iterations: u64, addr: *mut u64);
    fn read_8x2(iterations: u64, addr: *mut u64);
);
