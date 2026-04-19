use crate::unix_extern;

unix_extern!(
    fn read_4x3(iterations: u64, addr: *mut u64);
    fn read_8x3(iterations: u64, addr: *mut u64);
    fn read_16x3(iterations: u64, addr: *mut u64);
    fn read_32x3(iterations: u64, addr: *mut u64);

    fn read_same_32x1(iterations: u64, addr: *mut u64);
    fn read_same_32x2(iterations: u64, addr: *mut u64);
    fn read_same_32x3(iterations: u64, addr: *mut u64);
    fn read_same_32x4(iterations: u64, addr: *mut u64);

    fn write_32x1(iterations: u64, addr: *mut u64);
    fn write_32x2(iterations: u64, addr: *mut u64);
    fn write_32x3(iterations: u64, addr: *mut u64);
);
