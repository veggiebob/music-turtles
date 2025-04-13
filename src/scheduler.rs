use std::time::Duration;
use rodio::Source;
use rodio::source::SineWave;
use crate::composition::{Frequency, Instrument, Track, Volume};
use crate::player::Playable;
use crate::time::{MusicTime, Seconds, TimeSignature, BPM};

pub type Cursor = MusicTime;

pub struct Scheduler {
    pub bpm: BPM,
    pub time_signature: TimeSignature,
    pub tracks: Vec<(Track, Cursor)>,
    pub lookahead: MusicTime,
    pub looped: bool,
    pub loop_time: MusicTime,
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ScheduledSound {
    time: Seconds,
    duration: Seconds,
    volume: Volume,
    instrument: Instrument,
    pitch: Frequency
}

pub fn get_sine_source(length: Seconds, frequency: Frequency) -> impl Source<Item=f32> {
    let sources: Vec<Box<dyn Source<Item=f32> + Send>> = vec![
        Box::new(
            SineWave::new(frequency)
                .take_duration(Duration::from_secs_f32(length))
                .fade_in(Duration::from_millis(40))
        ),
        Box::new(
            SineWave::new(frequency).fade_out(Duration::from_millis(40))
        )
    ];

    rodio::source::from_iter(sources)
        .amplify((3.0 * 44.0 / frequency).clamp(0.0, 1.0))
}

impl Playable for ScheduledSound {
    /// start time, duration, and actual sound
    fn get_source(&self) -> (Seconds, Seconds, Box<dyn Source<Item=f32> + Send + 'static>) {
        let source = get_sine_source(self.duration, self.pitch);
        (
            self.time,
            self.duration,
            Box::new(source)
        )
    }
}

impl Scheduler {
    /// get the next events and update the cursors if necessary
    pub fn get_next_events_and_update(&mut self, current_track_pos: Seconds) -> Vec<ScheduledSound> {
        let mut current_music_time = MusicTime::from_seconds(self.time_signature, self.bpm, current_track_pos);
        let loop_end = self.loop_time;
        while current_music_time > loop_end {
            current_music_time = current_music_time.with(self.time_signature) - loop_end;
        }
        let loop_time_s = self.loop_time.to_seconds(self.time_signature, self.bpm);
        let mut end_music_time = current_music_time.with(self.time_signature) + self.lookahead;
        let end_non_looped = end_music_time;
        let looping = if self.looped && end_music_time > loop_end {
            while end_music_time > loop_end {
                end_music_time = end_music_time.with(self.time_signature) - loop_end;
            }
            true
        } else {
            false
        };
        let mut sounds = self.tracks.iter_mut()
            .map(|(track, cursor)| {
                let events = if looping {
                    if end_non_looped < *cursor {
                        vec![]
                    } else {
                        if *cursor <= end_music_time {
                            track.get_events_starting_between(*cursor, end_music_time, true)
                        } else {
                            let mut to_end = track.get_events_starting_between(*cursor, loop_end, true);
                            let from_beg = track.get_events_starting_between(MusicTime::zero(), end_music_time, false);
                            to_end.extend(from_beg);
                            to_end
                        }
                    }
                } else {
                    track.get_events_starting_between(*cursor, end_music_time, true)
                };
                *cursor = end_music_time;
                // make sure looped sounds happen afterward
                events.into_iter()
                    .map(|e| {
                        let start = e.start.to_seconds(self.time_signature, self.bpm);
                        let duration = e.duration.as_music_time(self.time_signature).to_seconds(self.time_signature, self.bpm);
                        let volume = e.volume;
                        let instrument = track.instrument;
                        ScheduledSound {
                            time: start,
                            duration,
                            volume,
                            instrument,
                            pitch: e.pitch.to_frequency(),
                        }
                    })
                    .map(|mut se| {
                        while se.time < current_track_pos {
                            se.time += loop_time_s;
                        }
                        se
                    }).collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();
        sounds.sort_by(|a: &ScheduledSound, b: &ScheduledSound| a.partial_cmp(b).unwrap());
        sounds
    }
}