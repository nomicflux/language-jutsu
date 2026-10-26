use std::error::Error;

use crate::{
    cli::{Mode, Request},
    format::Format,
    media, metadata, render, segmentation, sequence,
    source::{AcquiredSource, Source},
    transcription,
};

pub(crate) fn run(request: Request) -> Result<(), Box<dyn Error>> {
    request.source.validate()?;
    println!("Input: {}", request.source);
    println!("Language: {}", request.language);
    if matches!(&request.mode, Mode::Transcribe) {
        let language = request.language.code().to_owned();
        let input = acquire(request.source, request.save_source.as_deref())?;
        let decoded = decode(&input, request.language, request.window)?;
        let metadata =
            transcription::publish(&decoded, &language, &request.whisper, &request.output)?;
        println!("Transcript saved: {}", request.output.display());
        println!("Transcript metadata saved: {}", metadata.display());
        return Ok(());
    }
    let format = Format::load(&request.format_path, request.chunk_target_size_seconds)?;
    let formula = &format.playback;
    let settings = &format.chunking;
    if let Some(path) = &request.metadata_output {
        metadata::preflight(&request.output, path)?;
    }
    println!("Format: {}", request.format_path.display());
    println!("Chunk target size: {} seconds", settings.target_seconds());
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

    match &request.mode {
        Mode::Check => {
            println!(
                "Configuration checked. Audio contents have not been analyzed; no output was written."
            );
            if request.source.is_remote() {
                println!("Remote availability and recording duration have not been checked.");
            }
            Ok(())
        }
        Mode::Decode => {
            let input = acquire(request.source, request.save_source.as_deref())?;
            let decoded = decode(&input, request.language, request.window)?;
            let metadata = decoded.publish(&request.output)?;
            println!("Decoded PCM saved: {}", request.output.display());
            println!("Decoded metadata saved: {}", metadata.display());
            Ok(())
        }
        Mode::Segment | Mode::Sequence | Mode::Render => {
            let input = acquire(request.source, request.save_source.as_deref())?;
            let decoded = decode(&input, request.language, request.window)?;
            let analysis = segmentation::analyze(&decoded, settings)?;
            println!("Practice units: {}", analysis.units.len());
            let rate = f64::from(decoded.info().sample_rate.get());
            for (index, unit) in analysis.units.iter().enumerate() {
                println!(
                    "Unit {}: {:.3}–{:.3} seconds",
                    index + 1,
                    unit.start_frame as f64 / rate,
                    unit.end_frame as f64 / rate
                );
            }
            if matches!(&request.mode, Mode::Segment) {
                analysis.publish(decoded.info(), settings, &request.output)?;
                println!("Segmentation report saved: {}", request.output.display());
                Ok(())
            } else {
                let sequence = sequence::Sequence::build(analysis.units.len(), formula)?;
                let plan = sequence::Plan::build(&analysis, &format, decoded.info(), &sequence)?;
                println!("Practice groups: {}", sequence.group_count);
                println!("Planned plays: {}", sequence.plays().len());
                println!(
                    "Estimated listening duration: {:.3} seconds",
                    plan.estimated_seconds
                );
                if matches!(&request.mode, Mode::Sequence) {
                    plan.publish(&request.output)?;
                    println!("Sequence report saved: {}", request.output.display());
                    Ok(())
                } else {
                    let frames =
                        render::publish(&decoded, &analysis.units, &sequence, &request.output)?;
                    if let Some(path) = &request.metadata_output {
                        metadata::publish_render(
                            decoded.info(),
                            &format,
                            &plan,
                            frames,
                            &request.output,
                            path,
                        )?;
                        println!("Render metadata saved: {}", path.display());
                    }
                    println!("Rendered PCM frames: {frames}");
                    println!("Rendered PCM duration: {:.6} seconds", frames as f64 / rate);
                    println!("Study audio saved: {}", request.output.display());
                    Ok(())
                }
            }
        }
        Mode::Transcribe => unreachable!("transcription returns before format processing"),
    }
}

fn acquire(
    source: Source,
    save_source: Option<&std::path::Path>,
) -> Result<AcquiredSource, Box<dyn Error>> {
    let mut input = source.acquire()?;
    if let Some(path) = save_source {
        let sidecar = input.retain_download(path)?;
        println!("Downloaded source saved: {}", input.path().display());
        println!("Source provenance saved: {}", sidecar.display());
    }
    Ok(input)
}

fn decode(
    input: &AcquiredSource,
    language: crate::cli::LanguageSetting,
    window: crate::window::Request,
) -> Result<media::DecodedAudio, Box<dyn Error>> {
    let decoded = media::decode(input, language, window)?;
    let info = decoded.info();
    println!("Decoded sample rate: {} Hz", info.sample_rate);
    println!("Decoded channels: {}", info.channels);
    println!("Decoded frames: {}", info.frame_count);
    println!("Decoded duration: {:.6} seconds", info.duration_seconds());
    println!(
        "Selected source window: {:.6}–{:.6} seconds ({} frames)",
        info.selected_range.start_frame as f64 / f64::from(info.sample_rate.get()),
        info.selected_range.end_frame as f64 / f64::from(info.sample_rate.get()),
        info.selected_frame_count()
    );
    Ok(decoded)
}
