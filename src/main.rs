use std::fs::{File, OpenOptions, canonicalize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    file: String,
}

fn open_in_nvim(selected: String) {
    let status = Command::new("nvim")
        .arg(selected)
        .status()
        .expect("Failed to launch nvim");

    if !status.success() {
        eprintln!("nvim exited with error");
        std::process::exit(1);
    }
}

fn fuzzy_select(content: String, search: &str) -> Option<String> {
    for line in content.split("\n") {
        if line.contains(search) {
            return Some(line.to_string());
        }
    }
    None
}

fn main() {
    let HISTORY = format!("{}/.hopr_history", std::env::var("HOME").unwrap());

    let args = Cli::parse();
    let mut file = match File::open(&HISTORY) {
        Ok(file) => file,
        Err(_) => File::create(&HISTORY).unwrap(),
    };

    let mut buffer = vec![];
    let _ = file.read_to_end(&mut buffer);
    let content = String::from_utf8_lossy(&buffer);

    let selected = match fuzzy_select(content.to_string(), &args.file) {
        Some(v) => v,
        None => {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(HISTORY)
                .expect("Failed to open File");
            let path = PathBuf::from(&args.file);
            if let Ok(path) = canonicalize(&path) {
                let _ = writeln!(file, "{}", path.to_string_lossy());
                open_in_nvim(path.to_str().unwrap().to_string());
            }
            open_in_nvim(args.file);
            std::process::exit(1);
        }
    };

    open_in_nvim(selected);
}
