use std::error::Error;

use crate::{
    cli::{Mode, Request},
    formula::Formula,
};

pub(crate) fn run(request: Request, formula: Formula) -> Result<(), Box<dyn Error>> {
    let metadata = std::fs::metadata(&request.input)
        .map_err(|error| format!("cannot read input {}: {error}", request.input.display()))?;
    if !metadata.is_file() {
        return Err(format!("input is not a regular file: {}", request.input.display()).into());
    }

    println!("Input: {}", request.input.display());
    println!("Language: {}", request.language);
    println!("Formula: {}", request.formula_path.display());
    println!("Sequencing: {}", formula.sequencing());
    println!(
        "Speeds: {}",
        formula
            .speeds()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Joining group size: {}", formula.group_size());
    println!("Repetitions: {}", formula.repetitions());
    println!("Gap: {} ms", formula.gap().as_millis());
    println!("Output: {}", request.output.display());

    match request.mode {
        Mode::Check => {
            println!("Configuration checked. Audio contents have not been analyzed; no output was written.");
            Ok(())
        }
        Mode::Render => Err("audio segmentation, sequencing, and rendering are not implemented in this skeleton; use --check to validate configuration".into()),
    }
}
