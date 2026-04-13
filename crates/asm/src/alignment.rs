use crate::unix_extern;

unix_extern!(
    fn align_64(iterations: u64);
    fn misalign_1(iterations: u64);
    fn misalign_8(iterations: u64);
    fn misalign_16(iterations: u64);
    fn misalign_32(iterations: u64);
    fn misalign_48(iterations: u64);
    fn misalign_62(iterations: u64);
    fn misalign_63(iterations: u64);
);
