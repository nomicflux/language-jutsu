use std::{error::Error, io::Write, path::Path};

use tempfile::NamedTempFile;

use crate::{format::Format, media::DecodedInfo, sequence::Plan};

pub(crate) fn preflight(audio_output: &Path, metadata_output: &Path) -> Result<(), Box<dyn Error>> {
    fn destination(path: &Path) -> Result<std::path::PathBuf, Box<dyn Error>> {
        let parent = path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let name = path.file_name().ok_or("output must name a file")?;
        Ok(parent.canonicalize()?.join(name))
    }
    let audio = destination(audio_output)?;
    let metadata = destination(metadata_output)?;
    if audio == metadata {
        return Err("audio and metadata outputs must be different files".into());
    }
    if metadata.try_exists()? {
        return Err("render metadata output already exists; nothing was replaced".into());
    }
    NamedTempFile::new_in(metadata.parent().ok_or("metadata output has no parent")?)?;
    Ok(())
}

// Called only after this run has successfully published its new audio output.
pub(crate) fn publish_render(
    source: &DecodedInfo,
    format: &Format,
    plan: &Plan<'_>,
    rendered_frames: u64,
    audio_output: &Path,
    metadata_output: &Path,
) -> Result<(), Box<dyn Error>> {
    let result = write_render(
        source,
        format,
        plan,
        rendered_frames,
        audio_output,
        metadata_output,
    );
    if let Err(error) = result {
        std::fs::remove_file(audio_output).map_err(|rollback| {
            format!(
                "{error}; cannot remove incomplete render {}: {rollback}",
                audio_output.display()
            )
        })?;
        return Err(error);
    }
    Ok(())
}

fn write_render(
    source: &DecodedInfo,
    format: &Format,
    plan: &Plan<'_>,
    rendered_frames: u64,
    audio_output: &Path,
    metadata_output: &Path,
) -> Result<(), Box<dyn Error>> {
    let parent = metadata_output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = NamedTempFile::new_in(parent)?;
    let document = serde_json::json!({
        "schema_version": 1,
        "source": source,
        "format": format.document(),
        "sequence": {
            "group_count": plan.sequence.group_count,
            "play_count": plan.sequence.plays().len(),
            "estimated_duration_seconds": plan.estimated_seconds,
        },
        "output": {
            "path": audio_output.canonicalize()?,
            "sample_rate": source.sample_rate,
            "channels": source.channels,
            "pcm_frames": rendered_frames,
            "pcm_duration_seconds": rendered_frames as f64 / f64::from(source.sample_rate.get()),
        },
    });
    serde_json::to_writer_pretty(file.as_file_mut(), &document)?;
    writeln!(file)?;
    file.flush()?;
    file.persist_noclobber(metadata_output)
        .map_err(|error| format!("cannot save render metadata: {error}"))?;
    Ok(())
}
