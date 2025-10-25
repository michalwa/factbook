use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=prelude.pl");
    Command::new("swipl")
        .args([
            "-o",
            "target/state",
            "--stand_alone=false",
            "--autoload=false",
            // Provide `true` as an initalization goal to suppress the default banner
            "--goal=true",
            "-c",
            "prelude.pl",
        ])
        .status()
        .expect("could not build Prolog state");

    tauri_build::build()
}
