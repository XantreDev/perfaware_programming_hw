use crate::unix_extern;

unix_extern!(
    fn baseline_fill(len_dq: u64, src: *const u8, repeats: u64, dst: *mut u8);
    fn non_temporal_fill(len_dqw: u64, src: *const u8, repeats: u64, dst: *mut u8);
);
