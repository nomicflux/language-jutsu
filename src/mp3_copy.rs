use std::{error::Error, path::Path, process::Command};

use crate::{cli::Mp3Tags, render::run};

fn encoder(input: &Path, tags: &Mp3Tags, encoded: &Path) -> Command {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-nostdin", "-v", "error", "-xerror", "-y", "-i"])
        .arg(input)
        .args(["-map_metadata", "-1"]);
    let fields = [
        ("title", tags.title.clone()),
        ("album", tags.album.clone()),
        ("artist", tags.artist.clone()),
        ("track", tags.track.to_string()),
    ];
    for (key, value) in fields {
        command.arg("-metadata").arg(format!("{key}={value}"));
    }
    command
        .args(["-c:a", "libmp3lame", "-q:a", "2", "-f", "mp3"])
        .arg(encoded);
    command
}

pub(crate) fn copy(input: &Path, tags: &Mp3Tags, output: &Path) -> Result<(), Box<dyn Error>> {
    let (_, encoded) = crate::output::reserve(
        output,
        &["mp3"],
        "output must have an .mp3 extension",
        &format!("output already exists: {}", output.display()),
    )?;
    run(&mut encoder(input, tags, encoded.path()), "mp3 copy")?;
    encoded
        .persist_noclobber(output)
        .map_err(|error| format!("cannot save mp3 copy: {error}"))?;
    println!("Wrote {}", output.display());
    Ok(())
}
