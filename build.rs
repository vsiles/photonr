use std::env;
use std::path::Path;

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Note: dirty hack
        //       create a lib directory next to src/ and add the OpenCL.lib file in there
        // Note: If you prefer specifying the library path directly,
        // you can use the rustc-link-search and rustc-link-lib attributes in your Cargo.toml
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-search={}",
            Path::new(&dir).join("lib").display()
        );
    }
}
