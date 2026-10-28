use std::{error::Error, path::Path};

use tempfile::NamedTempFile;

fn extension(output: &Path, allowed: &[&str], wrong_type: &str) -> Result<String, Box<dyn Error>> {
    output
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| allowed.contains(value))
        .map(str::to_owned)
        .ok_or_else(|| wrong_type.into())
}

fn directory(output: &Path) -> &Path {
    output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

pub(crate) fn reserve(
    output: &Path,
    extensions: &[&str],
    wrong_type: &str,
    exists: &str,
) -> Result<(String, NamedTempFile), Box<dyn Error>> {
    let extension = extension(output, extensions, wrong_type)?;
    if output.try_exists()? {
        return Err(exists.into());
    }
    Ok((extension, NamedTempFile::new_in(directory(output))?))
}

#[cfg(test)]
mod tests {
    use super::reserve;

    fn attempt(name: &str) -> Result<(String, tempfile::NamedTempFile), String> {
        let root = std::env::temp_dir().join("language-jutsu-output-test");
        std::fs::create_dir_all(&root).unwrap();
        reserve(&root.join(name), &["mp3"], "wrong", "exists").map_err(|error| error.to_string())
    }

    #[test]
    fn accepts_only_lowercase_listed_extensions() {
        assert_eq!(attempt("new-copy.mp3").unwrap().0, "mp3");
        assert_eq!(attempt("new-copy.MP3").unwrap_err(), "wrong");
        assert_eq!(attempt("new-copy.wav").unwrap_err(), "wrong");
        assert_eq!(attempt("new-copy").unwrap_err(), "wrong");
    }

    #[test]
    fn refuses_an_existing_output_and_puts_the_temp_file_beside_it() {
        let root = std::env::temp_dir().join("language-jutsu-output-test");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("taken.mp3"), b"keep").unwrap();
        assert_eq!(attempt("taken.mp3").unwrap_err(), "exists");
        let (_, temp) = attempt("fresh.mp3").unwrap();
        assert_eq!(temp.path().parent().unwrap(), root);
    }
}
