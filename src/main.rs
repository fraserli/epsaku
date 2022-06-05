#![feature(backtrace)]

mod epub;
mod ui;

use epub::Epub;
use ui::Progress;

use std::backtrace::BacktraceStatus;
use std::collections::HashMap;
use std::fs::File;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// Path of EPUB file
    #[clap(required = true)]
    path: String,
}

fn read_history() -> Result<HashMap<String, Progress>> {
    let home = std::env::var("HOME")?;
    let path = format!("{home}/.local/share/epsaku/history.json");

    if let Ok(file) = File::open(&path) {
        let history: HashMap<String, Progress> = serde_json::from_reader(file)?;
        Ok(history)
    } else {
        Ok(HashMap::new())
    }
}

fn write_history(history: HashMap<String, Progress>) -> Result<()> {
    let home = std::env::var("HOME")?;
    let mut path = format!("{home}/.local/share/epsaku");
    std::fs::create_dir_all(&path)?;
    path.push_str("/history.json");

    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, &history)?;

    Ok(())
}

fn run(args: &Args) -> Result<()> {
    let mut epub = Epub::new(&args.path)?;

    let full_path = format!(
        "{}/{}",
        std::env::current_dir()?.to_str().unwrap(),
        &args.path
    );

    let mut history = read_history()?;

    let progress = ui::run(&mut epub, history.get(&full_path).copied())?;

    history.insert(full_path, progress);
    write_history(history)?;

    Ok(())
}

fn main() {
    let result = run(&Args::parse());
    match result {
        Ok(_) => {}
        Err(error) => {
            eprintln!("epsaku: {:#}", error);
            if error.backtrace().status() == BacktraceStatus::Captured {
                eprint!("\nStack backtrace:\n{}", error.backtrace());
            }
            std::process::exit(1);
        }
    }
}
