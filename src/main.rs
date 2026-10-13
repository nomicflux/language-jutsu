mod app;
mod cli;
mod format;
mod formula;
mod media;
mod metadata;
mod render;
mod segmentation;
mod sequence;
mod source;
mod transcription;
mod window;

use std::{error::Error, process::ExitCode};

use clap::Parser;

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), Box<dyn Error>> {
    let request = cli::Request::try_from(cli::Args::parse())?;
    app::run(request)
}
