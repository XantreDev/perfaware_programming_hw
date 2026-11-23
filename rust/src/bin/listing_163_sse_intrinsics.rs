#[unsafe(no_mangle)]
pub fn sqrt(value: f64) -> f64 {
    value.sqrt()
}

#[unsafe(no_mangle)]
pub fn intr_sqrt(value: f64) -> f64 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    unsafe {
        let reg = _mm_set_sd(value);
        let add = _mm_sqrt_sd(reg, reg);
        let result = _mm_cvtsd_f64(add);

        result
    }
}
