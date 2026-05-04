use crate::unix_extern;

unix_extern!(
    fn test_cache(iterations: u64, memory_mask: u64, buf: *mut u8);
);
