use std::{
    error::Error,
    fs::File,
    io::BufRead,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
const BINDINGS_PATH_KEY: &str = "RB_SYS_BINDINGS_PATH";
#[allow(dead_code)]
const CONFIG_PATH_KEY: &str = "RB_SYS_CARGO_CONFIG_PATH";

/// Use prebuilt bindings and cargo config.
#[allow(dead_code)]
pub fn run() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    // Copy the bindings to the output directory.
    let bindings_path = envvar(BINDINGS_PATH_KEY);
    let bindings_path = Path::new(&bindings_path);
    let bindings_file_name = bindings_path.file_name().unwrap();
    let bindings_output_path = out_dir.join(bindings_file_name);
    std::fs::copy(bindings_path, &bindings_output_path)?;
    println!(
        "cargo:rustc-env={}={}",
        BINDINGS_PATH_KEY,
        bindings_output_path.display()
    );

    let cargo_config_path = envvar(CONFIG_PATH_KEY);
    let cargo_config_path = Path::new(&cargo_config_path);
    let cargo_config_file_name = cargo_config_path.file_name().unwrap();
    let cargo_config_output_path = out_dir.join(cargo_config_file_name);
    std::fs::copy(cargo_config_path, &cargo_config_output_path)?;
    let cargo_config_output_file = File::open(&cargo_config_output_path)?;
    let cargo_config_output_file = std::io::BufReader::new(cargo_config_output_file);

    for line in cargo_config_output_file.lines() {
        let line = line?;
        if line.contains("/usr/local/rake-compiler") {
            eprintln!("skipping line: \"{line}\"");
            continue;
        }
        println!("{line}");
    }

    Ok(())
}

fn envvar(name: &str) -> String {
    if let Ok(var) = std::env::var(name) {
        println!("cargo:rerun-if-env-changed={name}");
        var
    } else {
        panic!("{} is not set", name);
    }
}
