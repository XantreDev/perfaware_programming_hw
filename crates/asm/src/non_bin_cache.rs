use crate::unix_extern;

unix_extern!(
    fn test_cache(iterations: u64, inner_iterations: u64, ptr: *mut u64);
);
pub const ITER_SIZE: usize = 256;
