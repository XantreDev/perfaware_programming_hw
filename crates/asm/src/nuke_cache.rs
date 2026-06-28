use crate::unix_extern;

unix_extern!(
    fn nuke_l1(iterations: u64, ptr: *mut u64);
);

pub const L1_ITERATION_READ_BYTES: usize = 32 * 4 * 4;
pub const L1_NUKE_REQUIRED_MEMORY: usize = 64 * 1024;
