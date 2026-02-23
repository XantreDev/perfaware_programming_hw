use std::process::ExitStatus;

impl ExpectedOk for ExitStatus {
    fn expect(self, msg: &str) {
        if !self.success() {
            panic!("{}", msg);
        }
    }
}
trait ExpectedOk {
    fn expect(self, msg: &str);
}

#[cfg(unix)]
fn main() {
    use std::env;
    use std::process::Command;

    let listing_path = "src/listing_139_code_alignment.asm";
    println!("cargo::rerun-if-changed={}", listing_path);
    let out_dir = env::var("OUT_DIR").unwrap();

    let out_file = &format!("{}/listing_139_code_alignment.o", out_dir);
    Command::new("nasm")
        .args(["-f", "elf64", &listing_path, "-o", out_file])
        .status()
        .expect("compiled to object file")
        .expect("packed");

    Command::new("ar")
        .args(["rcs", &format!("{}/libalign.a", out_dir), out_file])
        .status()
        .expect("packed correctly")
        .expect("packed");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=align");
}

#[cfg(not(unix))]
fn main() {
    println!("TODO implement build outside unix")
}
