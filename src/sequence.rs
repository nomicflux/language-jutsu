use std::{error::Error, io::Write, path::Path, time::Duration};

use serde::Serialize;
use tempfile::NamedTempFile;

use crate::{
    format::Format,
    formula::{Sequencing, Speed},
    media::DecodedInfo,
    segmentation::{Analysis, FrameRange},
};

#[derive(Serialize)]
pub(crate) struct Play<'a> {
    group_index: usize,
    unit_start: usize,
    unit_end_exclusive: usize,
    pub(crate) source_range: FrameRange,
    speed_index: usize,
    pub(crate) speed: &'a Speed,
    repetition_index: usize,
}

pub(crate) struct Plan<'a> {
    analysis: &'a Analysis,
    format: &'a Format,
    source: &'a DecodedInfo,
    pub(crate) group_count: usize,
    pub(crate) play_count: usize,
    pub(crate) estimated_seconds: f64,
}

impl<'a> Plan<'a> {
    pub(crate) fn build(
        analysis: &'a Analysis,
        format: &'a Format,
        source: &'a DecodedInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let formula = &format.playback;
        let units = &analysis.units;
        if units.is_empty() {
            return Err("cannot sequence an empty set of practice units".into());
        }
        for (index, unit) in units.iter().enumerate() {
            if unit.start_frame >= unit.end_frame
                || unit.start_frame < source.selected_range.start_frame
                || unit.end_frame > source.selected_range.end_frame
                || (index > 0 && units[index - 1].end_frame > unit.start_frame)
            {
                return Err(
                    "practice units must be nonempty, source-ordered, and within the decoded audio"
                        .into(),
                );
            }
        }
        let group_count = units.len().div_ceil(formula.group_size().get());
        let span_count = units
            .len()
            .checked_mul(2)
            .and_then(|count| count.checked_sub(group_count))
            .ok_or("joined span count overflow")?;
        let play_count = span_count
            .checked_mul(formula.speeds().len())
            .and_then(|count| count.checked_mul(formula.repetitions().get()))
            .ok_or("play count overflow")?;
        let mut span_frames = 0_u128;
        for group in units.chunks(formula.group_size().get()) {
            for (index, unit) in group.iter().enumerate() {
                span_frames = span_frames
                    .checked_add(u128::from(unit.end_frame - unit.start_frame))
                    .ok_or("source duration overflow")?;
                if index > 0 {
                    span_frames = span_frames
                        .checked_add(u128::from(unit.end_frame - group[0].start_frame))
                        .ok_or("joined duration overflow")?;
                }
            }
        }
        let speed_multiplier: f64 = formula
            .speeds()
            .iter()
            .map(|speed| 1.0 / speed.factor())
            .sum();
        let estimated_seconds = span_frames as f64 / f64::from(source.sample_rate.get())
            * speed_multiplier
            * formula.repetitions().get() as f64
            + (play_count - 1) as f64 * formula.gap().as_secs_f64();
        Duration::try_from_secs_f64(estimated_seconds).map_err(|error| {
            format!("estimated sequence duration is not representable: {error}")
        })?;
        Ok(Self {
            analysis,
            format,
            source,
            group_count,
            play_count,
            estimated_seconds,
        })
    }

    pub(crate) fn plays(&self) -> impl Iterator<Item = Play<'_>> {
        self.analysis
            .units
            .chunks(self.format.playback.group_size().get())
            .enumerate()
            .flat_map(move |(group_index, group)| {
                let span_count = group.len() * 2 - 1;
                let speed_count = self.format.playback.speeds().len();
                let group_start = group_index * self.format.playback.group_size().get();
                (0..span_count * speed_count).flat_map(move |pair| {
                    let (span_index, speed_index) = match self.format.playback.sequencing() {
                        Sequencing::TimingThenJoining => (pair % span_count, pair / span_count),
                        Sequencing::JoiningThenTiming => (pair / speed_count, pair % speed_count),
                    };
                    let (first, end) = if span_index == 0 {
                        (0, 1)
                    } else if span_index % 2 == 1 {
                        let first = span_index.div_ceil(2);
                        (first, first + 1)
                    } else {
                        (0, span_index / 2 + 1)
                    };
                    (0..self.format.playback.repetitions().get()).map(move |repetition_index| {
                        Play {
                            group_index,
                            unit_start: group_start + first,
                            unit_end_exclusive: group_start + end,
                            source_range: FrameRange {
                                start_frame: group[first].start_frame,
                                end_frame: group[end - 1].end_frame,
                            },
                            speed_index,
                            speed: &self.format.playback.speeds()[speed_index],
                            repetition_index,
                        }
                    })
                })
            })
    }

    pub(crate) fn publish(&self, output: &Path) -> Result<(), Box<dyn Error>> {
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = NamedTempFile::new_in(parent)?;
        // Stream occurrences rather than collecting a potentially large repeated sequence.
        write!(file, "{{\"segmentation\":")?;
        serde_json::to_writer(
            file.as_file_mut(),
            &self.analysis.document(self.source, &self.format.chunking),
        )?;
        write!(file, ",\"format\":")?;
        serde_json::to_writer(file.as_file_mut(), &self.format.document())?;
        write!(
            file,
            ",\"group_count\":{},\"play_count\":{},\"estimated_seconds\":{},\"plays\":[",
            self.group_count, self.play_count, self.estimated_seconds
        )?;
        let mut output_seconds = 0.0;
        for (index, play) in self.plays().enumerate() {
            if index > 0 {
                writeln!(file, ",")?;
            }
            let sound_seconds = (play.source_range.end_frame - play.source_range.start_frame)
                as f64
                / f64::from(self.source.sample_rate.get())
                / play.speed.factor();
            let following_gap = if index + 1 == self.play_count {
                Duration::ZERO
            } else {
                self.format.playback.gap()
            };
            let row = serde_json::json!({
                "index": index,
                "play": play,
                "estimated_start_seconds": output_seconds,
                "estimated_sound_seconds": sound_seconds,
                "following_gap_ms": following_gap.as_millis()
            });
            serde_json::to_writer(file.as_file_mut(), &row)?;
            output_seconds += sound_seconds + following_gap.as_secs_f64();
        }
        writeln!(file, "]}}")?;
        file.flush()?;
        file.persist_noclobber(output)
            .map_err(|error| format!("cannot save sequence report: {error}"))?;
        Ok(())
    }
}
