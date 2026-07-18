use crate::unix_extern;

unix_extern!(
    fn no_prefetching(len: u64, offsets: *const u32, ptr: *const u64) -> u64;
    fn prefetching(len: u64, offsets: *const u32, ptr: *const u64) -> u64;
);
