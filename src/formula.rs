use std::{fmt, num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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

#[derive(Clone, Copy, Serialize)]
#[serde(transparent)]
pub(crate) struct Speed(f64);

impl Speed {
    pub(crate) fn factor(&self) -> f64 {
        self.0
    }
}

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
    pub(crate) fn document(&self) -> serde_json::Value {
        serde_json::json!({
            "sequencing": self.sequencing,
            "segments": self.speeds,
            "joining": { "group_size": self.group_size.get() },
            "repetitions": self.repetitions.get(),
            "gap_ms": self.gap.as_millis()
        })
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
