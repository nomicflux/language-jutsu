mod anki;
mod app;
mod cli;
mod format;
mod formula;
mod media;
mod metadata;
mod model_cache;
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
    match args.command.take() {
        Some(cli::Command::Anki { input, output }) => anki::build(&input, &output),
        Some(cli::Command::DownloadModel { model }) => {
            let path = model_cache::fetch_model(&model)?;
            println!("Model cached: {}", path.display());
            Ok(())
        }
        None => app::run(cli::Request::try_from(args)?),
    }
}
