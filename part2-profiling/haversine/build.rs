#[cfg(target_arch = "aarch64")]
fn main() {
    // cc::Build::new()
    //     .cpp(true)
    //     .file("src/time/cpp/apple_arm_timer.hpp")
    //     .flag_if_supported("-std=c++11")
    //     .compile("arm_time");
}

#[cfg(not(target_arch = "aarch64"))]
fn main() {}
