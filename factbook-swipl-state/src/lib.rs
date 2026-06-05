use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn build(input_filename: &str, output_filename: &str) {
    println!("cargo::rerun-if-changed={input_filename}");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let output = match Command::new(project_dir.join("../deps/swipl/build/src/swipl"))
        .arg("-o")
        .arg(out_dir.join(output_filename))
        .args([
            "--stand_alone=false",
            "--autoload=false",
            // Provide `true` as an initalization goal to suppress the default banner
            "--goal=true",
            "-c",
            input_filename,
        ])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            println!("cargo::error=failed to run swipl: {e}");
            return;
        },
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        println!("cargo::error=could not build Prolog state:");
        for line in stderr.lines() {
            println!("cargo::error={line}");
        }
    }
}
