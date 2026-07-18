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
    path: String,
    name: String,
}
impl Listing {
    fn make(path: String, name: String) -> Listing {
        Listing { path, name }
    }
}

#[cfg(unix)]
fn main() {
    use std::path::Path;
    use std::process::Command;
    use std::{env, fs};

    let mut listings = Vec::with_capacity(64);
    for item in fs::read_dir(Path::new("src")).unwrap() {
        use std::ffi::OsStr;

        let unwrapped_path = item.unwrap().path();
        if unwrapped_path.is_file() && unwrapped_path.extension() == Some(OsStr::new("asm")) {
            let file_name = unwrapped_path.file_name().unwrap();
            let str_file_name = file_name.to_str().unwrap();
            let ext_idx = str_file_name.find(".").expect("must have extension");
            let name = str_file_name[0..ext_idx].replace("_", "");

            listings.push(Listing {
                name,
                path: unwrapped_path.to_str().unwrap().to_string(),
            })
        }
    }

    for item in listings {
        use std::path::Path;

        let listing_path = item.path;
        let filename = Path::new(&listing_path)
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
