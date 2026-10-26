mod anki;
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
    let mut args = cli::Args::parse();
    if let Some(cli::Command::Anki { input, output }) = args.command.take() {
        return anki::build(&input, &output);
    }
    let request = cli::Request::try_from(args)?;
    app::run(request)
}
