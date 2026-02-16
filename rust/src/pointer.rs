pub struct ParsedPointer {
    pub offset: u32,
    pub table_index: u32,
    pub directory_index: u32,
    pub directory_ptr_index: u32,
    pub pml4_index: u32,
    pub prefix: u32,
}
pub fn parse_ptr(ptr: u64) -> ParsedPointer {
    fn read_bits(value: u64, right_offset: u64, bits: u64) -> u64 {
        let mask = (1 << bits) - 1;
        (value & (mask << right_offset)) >> right_offset
    }

    ParsedPointer {
        offset: read_bits(ptr, 0, 12) as u32,
        table_index: read_bits(ptr, 12, 9) as u32,
        directory_index: read_bits(ptr, 12 + 9, 9) as u32,
        directory_ptr_index: read_bits(ptr, 12 + 9 * 2, 9) as u32,
        pml4_index: read_bits(ptr, 12 + 9 * 3, 9) as u32,
        prefix: read_bits(ptr, 12 + 9 * 4, 16) as u32,
    }
}
pub fn parse_raw_pointer(ptr: *const libc::c_void) -> ParsedPointer {
    parse_ptr(ptr as u64)
}
