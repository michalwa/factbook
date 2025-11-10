use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn build(input_filename: &str, output_filename: &str) {
    println!("cargo::rerun-if-changed={input_filename}");

    Command::new("swipl")
        .arg("-o")
        .arg(PathBuf::from(env::var("OUT_DIR").unwrap()).join(output_filename))
        .args([
            "--stand_alone=false",
            "--autoload=false",
            // Provide `true` as an initalization goal to suppress the default banner
            "--goal=true",
            "-c",
            input_filename,
        ])
        .status()
        .expect("could not build Prolog state");
}
