use dotenv::from_filename;

fn main() {
    from_filename(".env.local").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_dir = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Could not find workspace root")
        .join("lib");

    if !lib_dir.exists() {
        panic!("lib/ directory not found at {}", lib_dir.display());
    }
    let lib_dir = lib_dir.display();

    eprintln!("\x1b31mlib_dir:\x1b0m{lib_dir}");

    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=dylib=HSausmetrics-sdmx-0.1.0.0-inplace-ghc9.6.7");
    println!("cargo:rustc-link-lib=dylib=HSrts-ghc9.6.7");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
    println!(
        "cargo:rerun-if-changed={lib_dir}/libHSausmetrics-sdmx-0.1.0.0-inplace-ghc9.6.7.dylib"
    );
}
