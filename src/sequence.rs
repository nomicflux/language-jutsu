use std::{error::Error, io::Write, ops::Range, path::Path, time::Duration};

use serde::Serialize;
use tempfile::NamedTempFile;

use crate::{
    format::Format,
    formula::{Formula, Sequencing, Speed},
    media::DecodedInfo,
    segmentation::{Analysis, FrameRange},
};

#[derive(Serialize)]
pub(crate) struct Play {
    group_index: usize,
    unit_start: usize,
    unit_end_exclusive: usize,
    speed_index: usize,
    pub(crate) speed: Speed,
    repetition_index: usize,
    #[serde(skip)]
    pub(crate) following_gap: Duration,
}

impl Play {
    pub(crate) fn source_range(&self, units: &[FrameRange]) -> FrameRange {
        FrameRange {
            start_frame: units[self.unit_start].start_frame,
            end_frame: units[self.unit_end_exclusive - 1].end_frame,
        }
    }

    fn sound_seconds(&self, units: &[FrameRange], sample_rate: u32) -> f64 {
        let range = self.source_range(units);
        (range.end_frame - range.start_frame) as f64 / f64::from(sample_rate) / self.speed.factor()
    }
}

pub(crate) struct Sequence {
    plays: Vec<Play>,
    pub(crate) group_count: usize,
}

impl Sequence {
    pub(crate) fn build(unit_count: usize, formula: &Formula) -> Result<Self, Box<dyn Error>> {
        if unit_count == 0 {
            return Err("cannot sequence an empty set of practice units".into());
        }
        let group_size = formula.group_size().get();
        let mut groups = Vec::new();
        for start in (0..unit_count).step_by(group_size) {
            let mut spans = Vec::new();
            for unit in start..start.saturating_add(group_size).min(unit_count) {
                spans.push(unit..unit + 1);
                if unit > 0 && group_size > 1 {
                    spans.push((unit + 1).saturating_sub(group_size)..unit + 1);
                }
            }
            groups.push(spans);
        }
        let span_count: usize = groups.iter().map(Vec::len).sum();
        let capacity = span_count
            .checked_mul(formula.speeds().len())
            .and_then(|count| count.checked_mul(formula.repetitions().get()))
            .ok_or("play count overflow")?;
        let mut plays = Vec::new();
        plays.try_reserve_exact(capacity)?;
        for (group_index, spans) in groups.iter().enumerate() {
            let mut append = |span: &Range<usize>, speed_index: usize, speed: Speed| {
                for repetition_index in 0..formula.repetitions().get() {
                    plays.push(Play {
                        group_index,
                        unit_start: span.start,
                        unit_end_exclusive: span.end,
                        speed_index,
                        speed,
                        repetition_index,
                        following_gap: formula.gap(),
                    });
                }
            };
            match formula.sequencing() {
                Sequencing::TimingThenJoining => {
                    for (speed_index, &speed) in formula.speeds().iter().enumerate() {
                        for span in spans {
                            append(span, speed_index, speed);
                        }
                    }
                }
                Sequencing::JoiningThenTiming => {
                    for span in spans {
                        for (speed_index, &speed) in formula.speeds().iter().enumerate() {
                            append(span, speed_index, speed);
                        }
                    }
                }
            }
        }
        plays.last_mut().unwrap().following_gap = Duration::ZERO;
        Ok(Self {
            plays,
            group_count: groups.len(),
        })
    }

    pub(crate) fn plays(&self) -> &[Play] {
        &self.plays
    }
}

pub(crate) struct Plan<'a> {
    analysis: &'a Analysis,
    format: &'a Format,
    source: &'a DecodedInfo,
    pub(crate) sequence: &'a Sequence,
    pub(crate) estimated_seconds: f64,
}

impl<'a> Plan<'a> {
    pub(crate) fn build(
        analysis: &'a Analysis,
        format: &'a Format,
        source: &'a DecodedInfo,
        sequence: &'a Sequence,
    ) -> Result<Self, Box<dyn Error>> {
        let units = &analysis.units;
        if sequence.plays.last().map(|play| play.unit_end_exclusive) != Some(units.len()) {
            return Err("sequence unit count does not match the practice units".into());
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
        let estimated_seconds = sequence
            .plays()
            .iter()
            .map(|play| {
                play.sound_seconds(units, source.sample_rate.get())
                    + play.following_gap.as_secs_f64()
            })
            .sum();
        Duration::try_from_secs_f64(estimated_seconds).map_err(|error| {
            format!("estimated sequence duration is not representable: {error}")
        })?;
        Ok(Self {
            analysis,
            format,
            source,
            sequence,
            estimated_seconds,
        })
    }

    pub(crate) fn publish(&self, output: &Path) -> Result<(), Box<dyn Error>> {
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = NamedTempFile::new_in(parent)?;
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
            self.sequence.group_count,
            self.sequence.plays().len(),
            self.estimated_seconds
        )?;
        let mut output_seconds = 0.0;
        for (index, play) in self.sequence.plays().iter().enumerate() {
            if index > 0 {
                writeln!(file, ",")?;
            }
            let sound_seconds =
                play.sound_seconds(&self.analysis.units, self.source.sample_rate.get());
            let mut document = serde_json::to_value(play)?;
            document["source_range"] =
                serde_json::to_value(play.source_range(&self.analysis.units))?;
            let row = serde_json::json!({
                "index": index,
                "play": document,
                "estimated_start_seconds": output_seconds,
                "estimated_sound_seconds": sound_seconds,
                "following_gap_ms": play.following_gap.as_millis()
            });
            serde_json::to_writer(file.as_file_mut(), &row)?;
            output_seconds += sound_seconds + play.following_gap.as_secs_f64();
        }
        writeln!(file, "]}}")?;
        file.flush()?;
        file.persist_noclobber(output)
            .map_err(|error| format!("cannot save sequence report: {error}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_owns_expanded_plays_without_audio_or_a_retained_formula() {
        let sequence = {
            let formula: Formula = serde_json::from_value(serde_json::json!({
                "sequencing": "joining-then-timing",
                "segments": [0.7, 0.7, 1.0],
                "joining": { "group_size": 2 },
                "repetitions": 2,
                "gap_ms": 700
            }))
            .unwrap();
            Sequence::build(3, &formula).unwrap()
        };
        let plays = sequence.plays();
        assert_eq!(plays.len(), 30);
        assert_eq!(sequence.group_count, 2);
        let expected_spans = [(0, 0, 1), (0, 1, 2), (0, 0, 2), (1, 2, 3), (1, 1, 3)];
        for (plays, expected_span) in plays.chunks_exact(6).zip(expected_spans) {
            for play in plays {
                assert_eq!(
                    (play.group_index, play.unit_start, play.unit_end_exclusive),
                    expected_span
                );
            }
            let timings: Vec<_> = plays
                .iter()
                .map(|play| (play.speed_index, play.speed.factor(), play.repetition_index))
                .collect();
            assert_eq!(
                timings,
                [
                    (0, 0.7, 0),
                    (0, 0.7, 1),
                    (1, 0.7, 0),
                    (1, 0.7, 1),
                    (2, 1.0, 0),
                    (2, 1.0, 1)
                ]
            );
        }
        assert!(
            plays[..29]
                .iter()
                .all(|play| play.following_gap == Duration::from_millis(700))
        );
        assert_eq!(plays[29].following_gap, Duration::ZERO);
    }
}
