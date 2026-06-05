use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rustc-link-lib=swipl_static");
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=tinfo");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-search={}", project_dir.join("lib").display());

    let bindings = bindgen::builder()
        .header("src/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();

    factbook_swipl_state::build("prelude.pl", "state");
}
