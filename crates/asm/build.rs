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

struct Listing {
    path: &'static str,
    name: &'static str,
}
impl Listing {
    fn make(path: &'static str, name: &'static str) -> Listing {
        Listing { path, name }
    }
}

#[cfg(unix)]
fn main() {
    use std::env;
    use std::process::Command;

    let listings = [
        Listing::make("src/listing_139_code_alignment.asm", "align"),
        Listing::make("src/listing_141_load_store_ports.asm", "memports"),
        Listing::make("src/listing_142_simd.asm", "simdops"),
        Listing::make("src/listing_143_cache_size.asm", "cache"),
        Listing::make("src/listing_144_non_power_of_two", "nonbincache"),
    ];

    for item in listings {
        use std::path::Path;

        let listing_path = item.path;
        let filename = Path::new(listing_path)
            .file_name()
            .expect("must be valid")
            .to_str()
            .unwrap();

        println!("cargo::rerun-if-changed={}", listing_path);
        let out_dir = env::var("OUT_DIR").unwrap();

        let out_file = &format!("{}/{}.o", out_dir, filename);
        Command::new("nasm")
            .args(["-f", "elf64", &listing_path, "-o", out_file])
            .status()
            .expect("compiled to object file")
            .expect("packed");

        Command::new("ar")
            .args(["rcs", &format!("{}/lib{}.a", out_dir, item.name), out_file])
            .status()
            .expect("packed correctly")
            .expect("packed");

        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static={}", item.name);
    }
}

#[cfg(not(unix))]
fn main() {
    println!("TODO implement build outside unix")
}
