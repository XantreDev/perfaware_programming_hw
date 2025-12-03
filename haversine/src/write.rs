#[inline]
pub fn write_linear(slice: &mut [u8], seed: usize, until: usize) {
    for j in 0..until {
        slice[j] = (j + seed) as u8;
    }
}
#[inline]
pub fn write_backwards(slice: &mut [u8], seed: usize, until: usize) {
    let len = slice.len();
    for j in 0..until {
        slice[len - 1 - j] = (j + seed) as u8;
    }
}

pub struct RawAlloc(*mut libc::c_void, usize);
impl Drop for RawAlloc {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.0, self.1) };
    }
}
impl RawAlloc {
    pub fn new(bytes: usize) -> RawAlloc {
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        assert!(ptr != libc::MAP_FAILED);
        RawAlloc(ptr, bytes)
    }

    pub fn size(&self) -> usize {
        self.1
    }
    pub fn as_ptr(&self) -> *const libc::c_void {
        self.0
    }

    pub fn as_mut_ptr(&self) -> *mut libc::c_void {
        self.0
    }

    pub fn as_u8_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn as_u8_slice_mut<'a>(&'a self) -> &'a mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.0 as *mut u8, self.1) }
    }
}
