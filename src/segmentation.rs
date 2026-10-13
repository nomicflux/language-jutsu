use std::{
    error::Error,
    io::{BufReader, Read, Write},
    path::Path,
    str::FromStr,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::media::{DecodedAudio, DecodedInfo};
pub(crate) use crate::window::FrameRange;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "f64")]
pub(crate) struct ChunkTargetSize(f64);

impl TryFrom<f64> for ChunkTargetSize {
    type Error = String;

    fn try_from(seconds: f64) -> Result<Self, Self::Error> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err("chunk_target_size_seconds must be finite and positive".into());
        }
        let duration = Duration::try_from_secs_f64(seconds)
            .map_err(|_| "chunk_target_size_seconds is too large")?;
        if duration.is_zero() {
            return Err("chunk_target_size_seconds must be at least one nanosecond".into());
        }
        Ok(Self(seconds))
    }
}

impl FromStr for ChunkTargetSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<f64>()
            .map_err(|_| "chunk target must be a number of seconds".to_owned())?
            .try_into()
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    chunk_target_size_seconds: ChunkTargetSize,
}

impl Settings {
    pub(crate) fn apply_override(&mut self, target: Option<ChunkTargetSize>) {
        if let Some(target) = target {
            self.chunk_target_size_seconds = target;
        }
    }

    pub(crate) fn target_seconds(&self) -> f64 {
        self.chunk_target_size_seconds.0
    }
}

pub(crate) struct Analysis {
    quiet_intervals: Vec<FrameRange>,
    pub(crate) units: Vec<FrameRange>,
}

struct EnergyWindow {
    start_frame: u64,
    end_frame: u64,
    power: f64,
}

// Infer two local level groups; their midpoint separates quiet from active sound.
// Zero samples are handled independently so digital silence cannot dominate the fit.
fn adaptive_threshold(windows: &[EnergyWindow]) -> Option<f64> {
    let levels: Vec<_> = windows
        .iter()
        .filter(|window| window.power > 0.0)
        .map(|window| 10.0 * window.power.log10())
        .collect();
    let mut low = levels.iter().copied().reduce(f64::min)?;
    let mut high = levels.iter().copied().reduce(f64::max)?;
    for _ in 0..12 {
        let split = (low + high) / 2.0;
        let (mut low_sum, mut high_sum, mut low_count, mut high_count) = (0.0, 0.0, 0, 0);
        for &level in &levels {
            if level <= split {
                low_sum += level;
                low_count += 1;
            } else {
                high_sum += level;
                high_count += 1;
            }
        }
        if low_count == 0 || high_count == 0 {
            break;
        }
        let next_low = low_sum / f64::from(low_count);
        let next_high = high_sum / f64::from(high_count);
        let change = (next_low - low).abs() + (next_high - high).abs();
        low = next_low;
        high = next_high;
        if change < 0.001 {
            break;
        }
    }
    // Avoid interpreting small RMS fluctuations in a steady signal as pauses.
    (high - low >= 3.0).then(|| 10_f64.powf((low + high) / 20.0))
}

pub(crate) fn analyze(
    source: &DecodedAudio,
    settings: &Settings,
) -> Result<Analysis, Box<dyn Error>> {
    let info = source.info();
    let rate = info.sample_rate.get();
    let window_frames = u64::from(rate).div_ceil(100);
    let target_duration = Duration::try_from_secs_f64(settings.target_seconds())?;
    let target_frames =
        u64::try_from((target_duration.as_nanos() * u128::from(rate)).div_ceil(1_000_000_000))
            .map_err(|_| "chunk target exceeds the source frame index range")?;
    let minimum_unit = target_frames.div_ceil(3);
    let mut input = BufReader::new(source.open_selected_pcm()?);
    let mut energies = vec![0.0; usize::try_from(info.channels.get())?];
    let selected = &info.selected_range;
    let mut windows = Vec::new();
    let mut start = selected.start_frame;
    while start < selected.end_frame {
        let end = start.saturating_add(window_frames).min(selected.end_frame);
        energies.fill(0.0);
        for frame in start..end {
            for energy in &mut energies {
                let mut bytes = [0; 4];
                input.read_exact(&mut bytes)?;
                let sample = f64::from(f32::from_le_bytes(bytes));
                if !sample.is_finite() {
                    return Err(format!("non-finite PCM sample at source frame {frame}").into());
                }
                *energy += sample * sample;
            }
        }
        let power = energies.iter().copied().fold(0.0, f64::max) / (end - start) as f64;
        windows.push(EnergyWindow {
            start_frame: start,
            end_frame: end,
            power,
        });
        start = end;
    }
    if windows.iter().all(|window| window.power == 0.0) {
        return Err("selected audio has no nonzero signal".into());
    }

    let block = usize::try_from(target_frames.div_ceil(window_frames))?.max(100);
    let mut quiet = Vec::with_capacity(windows.len());
    for begin in (0..windows.len()).step_by(block) {
        let end = begin.saturating_add(block).min(windows.len());
        let context_start = begin.saturating_sub(block / 2);
        let context_end = end.saturating_add(block / 2).min(windows.len());
        let threshold = adaptive_threshold(&windows[context_start..context_end]);
        quiet.extend(windows[begin..end].iter().map(|window| {
            window.power == 0.0 || threshold.is_some_and(|threshold| window.power <= threshold)
        }));
    }
    let mut pending = Vec::new();
    let mut begin = 0;
    while begin < windows.len() {
        if !quiet[begin] {
            begin += 1;
            continue;
        }
        let mut end = begin + 1;
        while end < windows.len() && quiet[end] {
            end += 1;
        }
        pending.push((begin, end));
        begin = end;
    }
    let mut quiet_intervals = Vec::new();
    let mut candidates = Vec::new();
    while let Some((begin, end)) = pending.pop() {
        let range = FrameRange {
            start_frame: windows[begin].start_frame,
            end_frame: windows[end - 1].end_frame,
        };
        let mut confirmed = false;
        if begin > 0 && end < windows.len() {
            let left = windows[begin.saturating_sub(5)..begin]
                .iter()
                .map(|window| window.power)
                .fold(0.0, f64::max);
            let right = windows[end..end.saturating_add(5).min(windows.len())]
                .iter()
                .map(|window| window.power)
                .fold(0.0, f64::max);
            let surroundings = left.min(right);
            let peak = windows[begin..end]
                .iter()
                .map(|window| window.power)
                .fold(0.0, f64::max);
            let power = windows[begin..end]
                .iter()
                .map(|window| window.power)
                .sum::<f64>()
                / (end - begin) as f64;
            let depth = if surroundings > 0.0 {
                (surroundings - power) / surroundings
            } else {
                0.0
            };
            if peak < surroundings && depth >= 0.5 {
                confirmed = true;
                let midpoint = range.start_frame + (range.end_frame - range.start_frame) / 2;
                if selected.end_frame - midpoint >= minimum_unit {
                    let confidence = depth * (end - begin) as f64 / ((end - begin) as f64 + 2.0);
                    candidates.push((midpoint, confidence));
                }
            }
        }
        if !confirmed {
            // A quieter speaker may fall below the surrounding model's threshold.
            // Refit that region to find its own pauses instead of cutting its speech.
            if let Some(threshold) = adaptive_threshold(&windows[begin..end]) {
                let mut index = begin;
                while index < end {
                    if windows[index].power > threshold {
                        index += 1;
                        continue;
                    }
                    let first = index;
                    while index < end && windows[index].power <= threshold {
                        index += 1;
                    }
                    if first > begin || index < end {
                        pending.push((first, index));
                    }
                }
            }
        }
        quiet_intervals.push(range);
    }
    quiet_intervals.sort_by_key(|range| (range.start_frame, range.end_frame));
    candidates.sort_by_key(|&(midpoint, _)| midpoint);

    let mut boundaries = vec![selected.start_frame];
    let mut chunk_start = selected.start_frame;
    let mut cursor = 0;
    while chunk_start < selected.end_frame {
        while cursor < candidates.len() && candidates[cursor].0 - chunk_start < minimum_unit {
            cursor += 1;
        }
        let desired = chunk_start.saturating_add(target_frames);
        let mut cut = selected.end_frame;
        let mut best_score = cut.abs_diff(desired) as f64 / target_frames as f64;
        let mut winner = candidates.len();
        for (index, &(midpoint, confidence)) in candidates.iter().enumerate().skip(cursor) {
            let distance = midpoint.abs_diff(desired) as f64 / target_frames as f64;
            if midpoint > desired && distance > best_score {
                break;
            }
            let score = distance + 0.2 * (1.0 - confidence);
            if score < best_score || (score == best_score && midpoint < cut) {
                cut = midpoint;
                best_score = score;
                winner = index;
            }
        }
        boundaries.push(cut);
        chunk_start = cut;
        cursor = winner.saturating_add(1);
    }
    let units = boundaries
        .windows(2)
        .map(|pair| FrameRange {
            start_frame: pair[0],
            end_frame: pair[1],
        })
        .collect();
    Ok(Analysis {
        quiet_intervals,
        units,
    })
}

impl Analysis {
    pub(crate) fn document(&self, source: &DecodedInfo, settings: &Settings) -> serde_json::Value {
        let rate = f64::from(source.sample_rate.get());
        let units: Vec<_> = self
            .units
            .iter()
            .map(|unit| {
                serde_json::json!({
                    "start_frame": unit.start_frame,
                    "end_frame": unit.end_frame,
                    "start_seconds": unit.start_frame as f64 / rate,
                    "end_seconds": unit.end_frame as f64 / rate
                })
            })
            .collect();
        serde_json::json!({
            "source": source,
            "settings": settings,
            "quiet_intervals": self.quiet_intervals,
            "units": units
        })
    }

    pub(crate) fn publish(
        &self,
        source: &DecodedInfo,
        settings: &Settings,
        output: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let report = self.document(source, settings);
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(file.as_file_mut(), &report)?;
        writeln!(file)?;
        file.flush()?;
        file.persist_noclobber(output)
            .map_err(|error| format!("cannot save segmentation report: {error}"))?;
        Ok(())
    }
}
