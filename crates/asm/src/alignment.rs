#[cfg(unix)]
unsafe extern "C" {
    pub fn align_64(iterations: u64);
    pub fn misalign_1(iterations: u64);
    pub fn misalign_8(iterations: u64);
    pub fn misalign_16(iterations: u64);
    pub fn misalign_32(iterations: u64);
    pub fn misalign_48(iterations: u64);
    pub fn misalign_62(iterations: u64);
    pub fn misalign_63(iterations: u64);
}

#[cfg(not(unix))]
pub unsafe extern "C" fn align_64(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_1(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_8(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_16(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_32(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_48(iterations: u64) {
    panic!("TODO")
}

#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_62(iterations: u64) {
    panic!("TODO")
}
#[cfg(not(unix))]
pub unsafe extern "C" fn misalign_63(iterations: u64) {
    panic!("TODO")
}
