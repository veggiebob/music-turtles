use std::ops::{Add, Div};
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use enumkit::EnumValues;
use num::Integer;
use num::rational::Ratio;
use crate::time::{Beat, BeatUnit, MusicTime, TimeCompression, TimeSignature};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Serialize, Deserialize, EnumValues)]
pub enum Instrument {
    SineWave,
    Piano,
    Bass,
    // percussion
    BongoHigh,
    BongoLow,
    Shaker1,
    Shaker2,
}

impl Instrument {
    pub fn is_percussion(&self) -> bool {
        // matches!(self, Instrument::Drum | Instrument::Snare | Instrument::Cymbal)
        false
    }
    pub fn str_values() -> impl Iterator<Item=(Instrument, String)> {
        Instrument::values()
            .map(|i| (i, format!("{:?}", i)))
    }
}

/// [0, 12)
pub type NoteNum = u8;
pub type Octave = i8;

pub type Frequency = f32;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Pitch(pub Octave, pub NoteNum);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TrackId {
    Instrument(Instrument),
    Custom(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Track {
    pub identifier: TrackId,
    pub instrument: Instrument,
    pub events: Vec<Event>,
    pub rests: Vec<Event>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Event {
    pub start: MusicTime,
    pub duration: Beat,
    pub volume: Volume,
    pub pitch: Pitch,
}

pub const MAX_VOLUME: u32 = 100;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Volume(pub u32);

impl Volume {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32 / MAX_VOLUME as f32
    }
}

impl Event {
    pub fn get_end(&self, time_signature: TimeSignature) -> MusicTime {
        self.start.with(time_signature) + self.duration.as_music_time(time_signature)
    }
}

// weird that option doesn't work like this
fn min_option<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}
fn max_option<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

impl Track {
    pub fn visualize(&self, columns: usize, time_signature: TimeSignature) -> String {
        let mut s = String::new();
        s.push('[');
        let duration = self.get_duration(time_signature);
        let bpm = 1.;
        let total_time = duration.to_seconds(time_signature, bpm);
        for i in 0..columns {
            let time = (i as f32 / columns as f32) * total_time;
            let mt = MusicTime::from_seconds(time_signature, bpm, time);
            let evts = self.get_events_at(mt, time_signature);
            let rest_evts = self.get_rests_at(mt, time_signature);
            if evts.is_empty() {
                if rest_evts.is_empty() {
                    s.push(' ');
                } else {
                    s.push('-');
                }
            } else {
                if rest_evts.is_empty() {
                    s.push('X');
                } else {
                    s.push('?');
                }
            }
        }
        s.push(']');
        s
    }
    fn get_events_at(&self, time: MusicTime, time_signature: TimeSignature) -> Vec<Event> {
        self.events.iter()
            .filter(|e| time >= e.start && time <= e.get_end(time_signature))
            .map(|e| *e)
            .collect()
    }
    fn get_rests_at(&self, time: MusicTime, time_signature: TimeSignature) -> Vec<Event> {
        self.rests.iter()
            .filter(|e| time >= e.start && time <= e.get_end(time_signature))
            .map(|e| *e)
            .collect()
    }
    pub fn get_start(&self) -> Option<MusicTime> {
        min_option(self.events.iter()
                       .map(|e| e.start)
                       .min(), self.rests.iter()
                       .map(|e| e.start)
                       .min())
    }
    pub fn get_end(&self, time_signature: TimeSignature) -> Option<MusicTime> {
        max_option(self.events.iter()
                       .map(|e| e.get_end(time_signature))
                       .max(), self.rests.iter()
                       .map(|e| e.get_end(time_signature))
                       .max())
    }

    pub fn get_duration(&self, time_signature: TimeSignature) -> MusicTime {
        self.get_start()
            .map(|start| self.get_end(time_signature).map(
                |end|
                    end.with(time_signature) - start
            ))
            .flatten()
            .unwrap_or(MusicTime::zero())
    }

    /// End is always inclusive
    /// Doesn't include rests
    pub fn get_events_starting_between(&self, start: MusicTime, end: MusicTime, start_exclusive: bool) -> Vec<Event> {
        if (start_exclusive && start >= end) || start > end {
            return Vec::new();
        }
        let mut es = self.events.iter()
            .filter(|e| if start_exclusive {
                start < e.start
            } else {
                start <= e.start
            } && e.start <= end)
            .map(|e| *e)
            .collect::<Vec<_>>();
        es.sort();
        es
    }

    pub fn shift_by(&mut self, offset: MusicTime, time_signature: TimeSignature) {
        self.events.iter_mut()
            .chain(self.rests.iter_mut())
            .for_each(|e|
                e.start = e.start.with(time_signature) + offset
            );
    }

    pub fn transpose(&mut self, semitones: i8) {
        for event in &mut self.events {
            event.pitch.transpose(semitones);
        }
    }

    /// Flip entire track, keeping it within its start/end bounds.
    pub fn reverse(&mut self, time_signature: TimeSignature) {
        if let (Some(start), Some(end)) = (self.get_start(), self.get_end(time_signature)) {
            self.events.iter_mut()
                .chain(self.rests.iter_mut())
                .for_each(|e| {
                    let offset = e.start.with(time_signature) - start;
                    let new_start = (end.with(time_signature) - offset).with(time_signature) - e.duration.as_music_time(time_signature);
                    e.start = new_start;
                });
            self.events.reverse();
            self.rests.reverse();
        }
    }

    /// Compress all timings by the compression factor.
    /// Example: if the factor is 0.5, it will compress the track to half its length.
    pub fn compress(&mut self, time_signature: TimeSignature, compression: TimeCompression) {
        let factor = compression.0;
        if factor < Ratio::new(0, 1) {
            self.reverse(time_signature);
        }
        let factor = Ratio::new(factor.numer().abs() as BeatUnit, factor.denom().abs() as BeatUnit);
        if let (Some(start), Some(end)) = (self.get_start(), self.get_end(time_signature)) {
            self.events.iter_mut()
                .chain(self.rests.iter_mut())
                .for_each(|e| {
                    let offset = (e.start.with(time_signature) - start).with(time_signature) * factor;
                    e.start = start.with(time_signature) + offset.time;
                    e.duration = (e.duration.as_music_time(time_signature).with(time_signature) * factor).total_beats();
                });
        }
    }
}

impl Add<Self> for Track {
    type Output = Track;

    fn add(self, rhs: Self) -> Self::Output {
        if self.instrument != rhs.instrument {
            panic!("not the same instruments!");
        }
        let mut events = self.events;
        for event in rhs.events {
            events.push(event);
        }
        events.sort();
        let mut rests = self.rests;
        for rest in rhs.rests {
            rests.push(rest);
        }
        rests.sort();
        Track {
            identifier: self.identifier,
            instrument: self.instrument,
            events,
            rests,
        }
    }
}

impl Pitch {
    pub fn to_frequency(&self) -> Frequency {
        let Pitch(octave, note_num) = *self;
        let note_num = note_num as f32;
        let octave = octave as f32;
        let frequency = 440.0 * 2f32.powf(octave - 4. + (note_num - 9.0) / 12.0);
        frequency
    }
    pub fn to_midi_note(&self) -> u8 {
        let Pitch(octave, note_num) = *self;
        let note_num = note_num as u8;
        let octave = octave as u8;
        octave * 12 + note_num + 9
    }

    pub fn letter_name(&self) -> String {
        let Pitch(_, note_num) = *self;
        let note_num = note_num as u8;
        match note_num {
            0 => "A",
            1 => "Bb",
            2 => "B",
            3 => "C",
            4 => "C#",
            5 => "D",
            6 => "Eb",
            7 => "E",
            8 => "F",
            9 => "F#",
            10 => "G",
            11 => "Ab",
            _ => panic!("Invalid note number")
        }.to_string()
    }

    pub fn transpose(&mut self, semitones: i8) {
        let Pitch(octave, note_num) = *self;
        let new_note_num = (note_num as i8 + semitones).rem_euclid(12) as u8;
        let new_octave = octave + ((note_num as i8 + semitones) as f32 / 12.).floor() as i8;
        *self = Pitch(new_octave, new_note_num);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub tracks: Vec<Track>,
    pub time_signature: TimeSignature,
}

impl Composition {
    pub fn visualize(&self, columns: usize) -> String {
        let mut s = String::new();
        for track in &self.tracks {
            s.push_str(&track.visualize(columns, self.time_signature));
            s.push('\n');
        }
        s
    }
    pub fn get_duration(&self) -> MusicTime {
        let start = self.tracks.iter().filter_map(|t| t.get_start())
            .min();
        let end = self.tracks.iter().filter_map(|t| t.get_end(self.time_signature))
            .max();
        match (start, end) {
            (Some(start), Some(end)) => end.with(self.time_signature) - start,
            _ => MusicTime::zero()
        }
    }

    pub fn shift_by(&mut self, offset: MusicTime) {
        self.tracks.iter_mut()
            .for_each(|tr| tr.shift_by(offset, self.time_signature));
    }

    pub fn transpose(&mut self, semitones: i8) {
        for track in &mut self.tracks {
            track.transpose(semitones);
        }
    }

    /// Compress all timings by the compression factor toward the start of the track.
    /// If the factor is negative, it will reverse the track.
    /// Example, if the factor is 0.5, it will compress the track to half its length.
    pub fn compress(&mut self, compression: TimeCompression) {
        for track in &mut self.tracks {
            track.compress(self.time_signature, compression);
        }
    }
}

impl Add<Self> for Composition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.time_signature != rhs.time_signature {
            panic!("differing time signatures!!");
        }
        let mut map = HashMap::new();
        for track in self.tracks {
            let id = track.identifier;
            if let Some(mtrack) = map.remove(&id) {
                let new_track = mtrack + track;
                map.insert(id, new_track);
            } else {
                map.insert(id, track);
            }
        }
        Composition {
            tracks: map.into_values().collect(),
            time_signature: self.time_signature,
        }
    }
}

impl FromStr for Instrument {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "piano" => Ok(Instrument::Piano),
            s => {
                let instrument_enum: HashMap<_, _> = Instrument::str_values()
                    .map(|(i, i_name)| (i_name.to_ascii_lowercase(), i))
                    .collect();
                instrument_enum.get(s)
                    .map(|i| *i)
                    .ok_or(format!("Unknown instrument: {}", s))
            }
        }
    }
}

mod composition_element_tests {
    use num::rational::Ratio;
    use rodio::cpal::BufferSize::Default;
    use crate::composition::{Composition, Event, Instrument, Pitch, Track, TrackId, Volume};
    use crate::time::{Beat, MusicTime, TimeCompression, TimeSignature};

    fn assert_epsilon_close(a: f32, b: f32) {
        if (a - b).abs() < 0.01 {
            ()
        } else {
            panic!("left={} is not close to right={}", a, b);
        }
    }

    #[test]
    fn test_pitch_to_frequency_1() {
        let pitch = Pitch(4, 0); // C4
        let frequency = pitch.to_frequency();
        assert_epsilon_close(frequency, 261.63);
    }

    #[test]
    fn test_pitch_to_frequency_2() {
        let pitch = Pitch(3, 0); // C3
        let frequency = pitch.to_frequency();
        assert_epsilon_close(frequency, 261.63 / 2.);
    }

    #[test]
    fn test_transpose_1() {
        let mut pitch = Pitch(4, 0); // C4
        pitch.transpose(2);
        assert_eq!(pitch, Pitch(4, 2)); // D4
    }

    #[test]
    fn test_transpose_2() {
        let mut pitch = Pitch(4, 0); // C4
        pitch.transpose(-1);
        assert_eq!(pitch, Pitch(3, 11)); // B3
    }

    #[test]
    fn test_transpose_3() {
        let mut pitch = Pitch(4, 2); // D4
        pitch.transpose(-7);
        assert_eq!(pitch, Pitch(3, 7)); // G3
    }

    #[test]
    fn test_transpose_4() {
        let mut pitch = Pitch(4, 0); // C4
        pitch.transpose(12);
        assert_eq!(pitch, Pitch(5, 0)); // C5
    }

    fn comp_template(events: Vec<Event>) -> Composition {
        Composition {
            tracks: vec![
                Track {
                    identifier: TrackId::Custom(0),
                    instrument: Instrument::SineWave,
                    events,
                    rests: vec![],
                }
            ],
            time_signature: TimeSignature::common(),
        }
    }

    #[test]
    fn test_compression_1() {
        let compression = TimeCompression(Ratio::new(1, 2)); // 50% compression
        let mut composition1 = comp_template(vec![
            Event {
                start: MusicTime::measures(1),
                duration: Beat::whole(2),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            }
        ]);
        let composition_half = comp_template(vec![
            Event {
                start: MusicTime::measures(1),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            }
        ]);
        composition1.compress(compression);
        assert_eq!(composition1, composition_half);
    }

    #[test]
    fn test_compression_2() {
        let compression = TimeCompression(Ratio::new(-1, 1)); // -100% compression (reverse)
        let mut composition1 = comp_template(vec![
            Event {
                start: MusicTime::measures(1),
                duration: Beat::whole(2),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            }
        ]);
        let composition_reversed = comp_template(vec![
            Event {
                start: MusicTime::measures(1),
                duration: Beat::whole(2),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            }
        ]);
        composition1.compress(compression);
        assert_eq!(composition1, composition_reversed);
    }

    #[test]
    fn test_compression_3() {
        let compression = TimeCompression(Ratio::new(-1, 1)); // -100% compression (reverse)
        let mut composition1 = comp_template(vec![
            Event {
                start: MusicTime(1, Beat::whole(0)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            },
            Event {
                start: MusicTime(1, Beat::whole(1)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 1),
            }
        ]);
        let composition_reversed = comp_template(vec![
            Event {
                start: MusicTime(1, Beat::whole(0)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 1),
            },
            Event {
                start: MusicTime(1, Beat::whole(1)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            }
        ]);
        composition1.compress(compression);
        assert_eq!(composition1, composition_reversed);
    }

    #[test]
    fn test_compression_4() {
        let compression = TimeCompression(Ratio::new(1, 2)); // 50% compression
        let mut composition1 = comp_template(vec![
            Event {
                start: MusicTime(1, Beat::whole(0)),
                duration: Beat::whole(2),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            },
            Event {
                start: MusicTime(1, Beat::whole(2)),
                duration: Beat::whole(2),
                volume: Volume(100),
                pitch: Pitch(4, 1),
            }
        ]);
        let composition_half = comp_template(vec![
            Event {
                start: MusicTime(1, Beat::whole(0)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 0),
            },
            Event {
                start: MusicTime(1, Beat::whole(1)),
                duration: Beat::whole(1),
                volume: Volume(100),
                pitch: Pitch(4, 1),
            }
        ]);
        composition1.compress(compression);
        assert_eq!(composition1, composition_half);
    }
}