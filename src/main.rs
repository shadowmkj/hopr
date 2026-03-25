use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use clap::Parser;
use hopr::db::Database;

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

fn open_in_nvim_unix(selected: String, dir: Option<&Path>) {
    if let Some(d) = dir {
        let _ = Command::new("nvim").arg(selected).current_dir(d).exec();
    } else {
        let _ = Command::new("nvim").arg(selected).exec();
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

fn main() -> Result<()> {
    let args = Cli::parse();
    let history = format!("{}/.hopr_history", std::env::var("HOME").unwrap());
    let mut database = Database::new(history.into());
    let mut database = match database.load() {
        Ok(v) => v,
        Err(_) => database,
    };

    if args.file == "list" {
        println!("{}", database);
        return Ok(());
    }

    println!("{:?}", database);
    let selected = database.query(&args.file);
    println!("{:?}", selected);

    let mut dir: Option<&Path> = Option::None;
    let to_open = match selected {
        Ok(v) => {
            let path = v.path.to_string_lossy();
            dir = v.path.parent();
            path
        }
        Err(_) => args.file.into(),
    };

    open_in_nvim_unix(to_open.to_string(), dir);
    Ok(())
}
