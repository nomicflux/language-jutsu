use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;

use crate::{
    media::DecodedAudio,
    segmentation::FrameRange,
    sequence::{Play, Sequence},
};

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

pub(crate) fn run(command: &mut Command, stage: &str) -> Result<(), Box<dyn Error>> {
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

const MP3_RATES: [u32; 9] = [8000, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000];

fn frame_bytes(source: &DecodedAudio) -> u64 {
    u64::from(source.info().channels.get()) * 4
}

fn check_mp3_compatible(source: &DecodedAudio, format: &str) -> Result<(), Box<dyn Error>> {
    let info = source.info();
    if format == "mp3" && (info.channels.get() > 2 || !MP3_RATES.contains(&info.sample_rate.get()))
    {
        return Err("MP3 cannot preserve this source's sample rate/channel count; use .wav".into());
    }
    Ok(())
}

fn gap_bytes(source: &DecodedAudio, play: &Play) -> Result<u64, Box<dyn Error>> {
    let rate = u128::from(source.info().sample_rate.get());
    let gap_frames = u64::try_from((play.following_gap.as_nanos() * rate).div_ceil(1_000_000_000))
        .map_err(|_| "practice gap exceeds the frame index range")?;
    Ok(gap_frames
        .checked_mul(frame_bytes(source))
        .ok_or("practice gap byte count overflow")?)
}

fn stretch(
    source: &DecodedAudio,
    clip: &Path,
    factor: f64,
    index: usize,
) -> Result<NamedTempFile, Box<dyn Error>> {
    let stretched = NamedTempFile::new()?;
    let mut command = pcm_input(source, clip);
    command
        .args([
            "-af",
            &tempo_filter(factor),
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
    if bytes == 0 || bytes % frame_bytes(source) != 0 {
        return Err(format!(
            "tempo processing produced empty or incomplete PCM at play {}",
            index + 1
        )
        .into());
    }
    Ok(stretched)
}

fn append_play(
    source: &DecodedAudio,
    units: &[FrameRange],
    play: &Play,
    index: usize,
    assembled: &mut File,
) -> Result<(), Box<dyn Error>> {
    let mut input = source.open_range_pcm(&play.source_range(units))?;
    if play.speed.factor() == 1.0 {
        io::copy(&mut input, assembled)?;
        return Ok(());
    }
    let mut clip = NamedTempFile::new()?;
    io::copy(&mut input, clip.as_file_mut())?;
    clip.flush()?;
    let stretched = stretch(source, clip.path(), play.speed.factor(), index)?;
    io::copy(&mut File::open(stretched.path())?, assembled)?;
    Ok(())
}

fn assemble(
    source: &DecodedAudio,
    units: &[FrameRange],
    sequence: &Sequence,
) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut assembled = NamedTempFile::new()?;
    for (index, play) in sequence.plays().iter().enumerate() {
        let gap = gap_bytes(source, play)?;
        append_play(source, units, play, index, assembled.as_file_mut())?;
        io::copy(&mut io::repeat(0).take(gap), assembled.as_file_mut())?;
    }
    assembled.flush()?;
    Ok(assembled)
}

fn encode(
    source: &DecodedAudio,
    assembled: &Path,
    format: &str,
    encoded: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut command = pcm_input(source, assembled);
    command.args(["-y", "-c:a"]);
    if format == "wav" {
        command.args(["pcm_f32le", "-rf64", "auto"]);
    } else {
        command.args(["libmp3lame", "-q:a", "2"]);
    }
    command
        .arg("-f")
        .arg(format)
        .arg(encoded)
        .stdout(Stdio::null());
    run(&mut command, "study encoding")?;
    if std::fs::metadata(encoded)?.len() == 0 {
        return Err("study encoder produced an empty file".into());
    }
    Ok(())
}

pub(crate) fn publish(
    source: &DecodedAudio,
    units: &[FrameRange],
    sequence: &Sequence,
    output: &Path,
) -> Result<u64, Box<dyn Error>> {
    let (format, encoded) = crate::output::reserve(
        output,
        &["wav", "mp3"],
        "study output must have a .wav or .mp3 extension",
        "study output already exists; nothing was replaced",
    )?;
    check_mp3_compatible(source, &format)?;
    let assembled = assemble(source, units, sequence)?;
    encode(source, assembled.path(), &format, encoded.path())?;
    encoded
        .persist_noclobber(output)
        .map_err(|error| format!("cannot save study audio: {error}"))?;
    Ok(assembled.as_file().metadata()?.len() / frame_bytes(source))
}
