use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;

use crate::{formula::Formula, media::DecodedAudio, sequence::Plan};

// Keep every stage inside atempo's blending range; their product is the requested speed.
fn tempo_filter(mut factor: f64) -> String {
    let mut stages = Vec::new();
    while factor < 0.5 {
        stages.push("atempo=0.5".to_owned());
        factor *= 2.0;
    }
    while factor > 2.0 {
        stages.push("atempo=2".to_owned());
        factor /= 2.0;
    }
    if factor != 1.0 {
        stages.push(format!("atempo={factor}"));
    }
    stages.join(",")
}

fn pcm_input(source: &DecodedAudio, path: &Path) -> Command {
    let info = source.info();
    let mut command = Command::new("ffmpeg");
    command
        .args(["-nostdin", "-v", "error", "-xerror", "-f", "f32le", "-ar"])
        .arg(info.sample_rate.to_string())
        .arg("-ac")
        .arg(info.channels.to_string());
    if let Some(layout) = info.channel_layout() {
        command.arg("-channel_layout").arg(layout);
    }
    command
        .arg("-i")
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::piped());
    command
}

fn run(command: &mut Command, stage: &str) -> Result<(), Box<dyn Error>> {
    let result = command
        .output()
        .map_err(|error| format!("cannot start ffmpeg for {stage}: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "ffmpeg {stage} failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .into());
    }
    Ok(())
}

pub(crate) fn publish(
    source: &DecodedAudio,
    plan: &Plan<'_>,
    formula: &Formula,
    output: &Path,
) -> Result<u64, Box<dyn Error>> {
    if output.try_exists()? {
        return Err("study output already exists; nothing was replaced".into());
    }
    let format = output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if format != "wav" && format != "mp3" {
        return Err("study output must have a .wav or .mp3 extension".into());
    }
    let info = source.info();
    let rate = info.sample_rate.get();
    if format == "mp3"
        && (info.channels.get() > 2
            || ![8000, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000].contains(&rate))
    {
        return Err("MP3 cannot preserve this source's sample rate/channel count; use .wav".into());
    }
    let frame_bytes = u64::from(info.channels.get()) * 4;
    let gap_frames =
        u64::try_from((formula.gap().as_nanos() * u128::from(rate)).div_ceil(1_000_000_000))
            .map_err(|_| "practice gap exceeds the frame index range")?;
    let gap_bytes = gap_frames
        .checked_mul(frame_bytes)
        .ok_or("practice gap byte count overflow")?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let encoded = NamedTempFile::new_in(parent)?;
    let mut assembled = NamedTempFile::new()?;
    for (index, play) in plan.plays().enumerate() {
        let mut input = source.open_range_pcm(&play.source_range)?;
        if play.speed.factor() == 1.0 {
            io::copy(&mut input, assembled.as_file_mut())?;
        } else {
            let mut clip = NamedTempFile::new()?;
            io::copy(&mut input, clip.as_file_mut())?;
            clip.flush()?;
            let stretched = NamedTempFile::new()?;
            let mut command = pcm_input(source, clip.path());
            command
                .args([
                    "-af",
                    &tempo_filter(play.speed.factor()),
                    "-c:a",
                    "pcm_f32le",
                    "-f",
                    "f32le",
                    "pipe:1",
                ])
                .stdout(Stdio::from(stretched.as_file().try_clone()?));
            run(
                &mut command,
                &format!("tempo processing at play {}", index + 1),
            )?;
            let bytes = stretched.as_file().metadata()?.len();
            if bytes == 0 || bytes % frame_bytes != 0 {
                return Err(format!(
                    "tempo processing produced empty or incomplete PCM at play {}",
                    index + 1
                )
                .into());
            }
            io::copy(&mut File::open(stretched.path())?, assembled.as_file_mut())?;
        }
        if index + 1 < plan.play_count && gap_bytes > 0 {
            io::copy(&mut io::repeat(0).take(gap_bytes), assembled.as_file_mut())?;
        }
    }
    assembled.flush()?;
    let mut command = pcm_input(source, assembled.path());
    command.args(["-y", "-c:a"]);
    if format == "wav" {
        command.args(["pcm_f32le", "-rf64", "auto"]);
    } else {
        command.args(["libmp3lame", "-q:a", "2"]);
    }
    command
        .arg("-f")
        .arg(&format)
        .arg(encoded.path())
        .stdout(Stdio::null());
    run(&mut command, "study encoding")?;
    if encoded.as_file().metadata()?.len() == 0 {
        return Err("study encoder produced an empty file".into());
    }
    encoded
        .persist_noclobber(output)
        .map_err(|error| format!("cannot save study audio: {error}"))?;
    Ok(assembled.as_file().metadata()?.len() / frame_bytes)
}
