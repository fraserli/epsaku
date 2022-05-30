#![feature(backtrace)]

mod epub;
mod ui;

use epub::Epub;

use std::backtrace::BacktraceStatus;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// Path of EPUB file
    #[clap(required = true)]
    path: String,
}

fn run(args: &Args) -> Result<()> {
    let mut epub = Epub::new(&args.path)?;

    ui::run(&mut epub)?;

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
