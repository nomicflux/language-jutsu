use std::{
    error::Error, fmt, fs::File, io::BufReader, num::NonZeroUsize, path::Path, time::Duration,
};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Sequencing {
    TimingThenJoining,
    JoiningThenTiming,
}

impl fmt::Display for Sequencing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TimingThenJoining => "timing-then-joining (speed first)",
            Self::JoiningThenTiming => "joining-then-timing (span first)",
        })
    }
}

pub(crate) struct Speed(f64);

impl fmt::Display for Speed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}x", self.0)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJoining {
    group_size: NonZeroUsize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFormula {
    sequencing: Sequencing,
    segments: Vec<f64>,
    joining: RawJoining,
    repetitions: NonZeroUsize,
    gap_ms: u64,
}

#[derive(Deserialize)]
#[serde(try_from = "RawFormula")]
pub(crate) struct Formula {
    sequencing: Sequencing,
    speeds: Vec<Speed>,
    group_size: NonZeroUsize,
    repetitions: NonZeroUsize,
    gap: Duration,
}

impl TryFrom<RawFormula> for Formula {
    type Error = String;

    fn try_from(raw: RawFormula) -> Result<Self, Self::Error> {
        if raw.segments.is_empty() {
            return Err("segments must contain at least one speed".into());
        }
        let speeds = raw
            .segments
            .into_iter()
            .enumerate()
            .map(|(index, speed)| {
                if speed.is_finite() && speed > 0.0 {
                    Ok(Speed(speed))
                } else {
                    Err(format!(
                        "segments[{index}] must be a finite, positive speed"
                    ))
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            sequencing: raw.sequencing,
            speeds,
            group_size: raw.joining.group_size,
            repetitions: raw.repetitions,
            gap: Duration::from_millis(raw.gap_ms),
        })
    }
}

impl Formula {
    pub(crate) fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)
            .map_err(|error| format!("cannot open formula {}: {error}", path.display()))?;
        serde_json::from_reader(BufReader::new(file))
            .map_err(|error| format!("invalid formula {}: {error}", path.display()).into())
    }

    pub(crate) fn sequencing(&self) -> &Sequencing {
        &self.sequencing
    }

    pub(crate) fn speeds(&self) -> &[Speed] {
        &self.speeds
    }

    pub(crate) fn group_size(&self) -> NonZeroUsize {
        self.group_size
    }

    pub(crate) fn repetitions(&self) -> NonZeroUsize {
        self.repetitions
    }

    pub(crate) fn gap(&self) -> Duration {
        self.gap
    }
}
