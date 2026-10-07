mod app;
mod cli;
mod formula;

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
    let formula = formula::Formula::load(&request.formula_path)?;
    app::run(request, formula)
}
