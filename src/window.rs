use std::{str::FromStr, time::Duration};

use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct Timestamp(Duration);

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        fn integer(value: &str) -> Result<u64, String> {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err("time components must contain nonnegative decimal digits".into());
            }
            value
                .parse()
                .map_err(|_| "time component is too large".into())
        }
        let parts: Vec<_> = value.split(':').collect();
        if parts.len() > 3 {
            return Err("use seconds, MM:SS, or HH:MM:SS".into());
        }
        let seconds_part = parts[parts.len() - 1];
        let (seconds, fraction) = match seconds_part.split_once('.') {
            Some((seconds, fraction)) => (seconds, Some(fraction)),
            None => (seconds_part, None),
        };
        let seconds = integer(seconds)?;
        if parts.len() > 1 && seconds >= 60 {
            return Err("seconds in a colon-formatted time must be less than 60".into());
        }
        let prefix = match parts.as_slice() {
            [_] => 0,
            [minutes, _] => integer(minutes)?
                .checked_mul(60)
                .ok_or("time is too large")?,
            [hours, minutes, _] => {
                let minutes = integer(minutes)?;
                if minutes >= 60 {
                    return Err("minutes in HH:MM:SS must be less than 60".into());
                }
                integer(hours)?
                    .checked_mul(3600)
                    .and_then(|hours| hours.checked_add(minutes * 60))
                    .ok_or("time is too large")?
            }
            _ => return Err("use seconds, MM:SS, or HH:MM:SS".into()),
        };
        let nanos = match fraction {
            None => 0,
            Some(fraction) => {
                if fraction.is_empty() || fraction.len() > 9 {
                    return Err("fractional seconds require 1 to 9 decimal digits".into());
                }
                u32::try_from(integer(fraction)?).map_err(|_| "invalid fractional seconds")?
                    * 10_u32.pow(9 - fraction.len() as u32)
            }
        };
        Ok(Self(Duration::new(
            prefix.checked_add(seconds).ok_or("time is too large")?,
            nanos,
        )))
    }
}

#[derive(Serialize)]
pub(crate) struct FrameRange {
    pub(crate) start_frame: u64,
    pub(crate) end_frame: u64,
}

#[derive(Clone, Serialize)]
pub(crate) struct Request {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

impl Request {
    pub(crate) fn new(start: Option<Timestamp>, end: Option<Timestamp>) -> Result<Self, String> {
        if let (Some(start), Some(end)) = (&start, &end)
            && start.0 >= end.0
        {
            return Err("--start must be less than --end".into());
        }
        Ok(Self { start, end })
    }

    pub(crate) fn resolve(
        &self,
        sample_rate: u32,
        total_frames: u64,
    ) -> Result<FrameRange, String> {
        fn boundary(time: &Timestamp, rate: u32, total: u64, name: &str) -> Result<u64, String> {
            let scaled = time.0.as_nanos() * u128::from(rate);
            if scaled > u128::from(total) * 1_000_000_000 {
                return Err(format!("--{name} is beyond the decoded recording's end"));
            }
            u64::try_from(scaled.div_ceil(1_000_000_000))
                .map_err(|_| "time exceeds source frame index range".into())
        }
        let start_frame = self
            .start
            .as_ref()
            .map(|time| boundary(time, sample_rate, total_frames, "start"))
            .transpose()?
            .unwrap_or(0);
        let end_frame = self
            .end
            .as_ref()
            .map(|time| boundary(time, sample_rate, total_frames, "end"))
            .transpose()?
            .unwrap_or(total_frames);
        if start_frame >= end_frame {
            return Err(
                "selected window contains no sample frames; start must be before end".into(),
            );
        }
        Ok(FrameRange {
            start_frame,
            end_frame,
        })
    }
}
