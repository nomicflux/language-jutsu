use std::{error::Error, path::Path};

use tempfile::NamedTempFile;

pub(crate) fn reserve(
    output: &Path,
    extensions: &[&str],
    wrong_type: &str,
    exists: &str,
) -> Result<(String, NamedTempFile), Box<dyn Error>> {
    let extension = output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !extensions.contains(&extension.as_str()) {
        return Err(wrong_type.into());
    }
    if output.try_exists()? {
        return Err(exists.into());
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok((extension, NamedTempFile::new_in(parent)?))
}
