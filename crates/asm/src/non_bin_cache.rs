use crate::unix_extern;

unix_extern!(
    fn test_cache_non_bin(iterations: u64, inner_reads: u64, ptr: *mut u64);
);

pub const READ_SIZE: usize = 128;
