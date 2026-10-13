use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

use crate::{
    formula::Formula,
    segmentation::{ChunkTargetSize, Settings},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Format {
    pub(crate) chunking: Settings,
    pub(crate) playback: Formula,
}

impl Format {
    pub(crate) fn load(
        path: &Path,
        target_override: Option<ChunkTargetSize>,
    ) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)
            .map_err(|error| format!("cannot open format {}: {error}", path.display()))?;
        let mut format: Self = serde_json::from_reader(BufReader::new(file))
            .map_err(|error| format!("invalid format {}: {error}", path.display()))?;
        format.chunking.apply_override(target_override);
        Ok(format)
    }

    pub(crate) fn document(&self) -> serde_json::Value {
        serde_json::json!({"chunking": self.chunking, "playback": self.playback.document()})
    }
}
