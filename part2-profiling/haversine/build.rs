#[cfg(target_arch = "aarch64")]
fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/time/cpp/apple_arm_timer.cpp")
        .flag_if_supported("-std=c++11")
        .compile("arm_time");
    println!("cargo:rerun-if-changed=src/time/cpp/apple_arm_timer.hpp");
    println!("cargo:rerun-if-changed=src/time/cpp/apple_arm_timer.cpp");
    println!("cargo:rerun-if-changed=src/time/cpp/apple_arm_events.hpp");
}

#[cfg(not(target_arch = "aarch64"))]
fn main() {}
