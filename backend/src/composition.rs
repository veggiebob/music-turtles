use std::ops::Add;
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use enumkit::EnumValues;
use crate::time::{Beat, MusicTime, TimeSignature};

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
            // todo: fix!!!
            let Pitch(octave, note_num) = event.pitch;
            let new_note_num = (note_num as i8 + semitones).rem_euclid(12) as NoteNum;
            let new_octave = octave + (note_num as i8 + semitones - 12) / 12;
            event.pitch = Pitch(new_octave, new_note_num);
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
}

#[derive(Clone, Debug)]
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