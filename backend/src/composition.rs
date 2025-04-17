use std::ops::Add;
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::time::{Beat, MusicTime, TimeSignature};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Serialize, Deserialize)]
pub enum Instrument {
    SineWave
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

impl Track {
    pub fn get_start(&self) -> Option<MusicTime> {
        self.events.iter()
            .map(|e| e.start)
            .min()
    }
    pub fn get_end(&self, time_signature: TimeSignature) -> Option<MusicTime> {
        self.events.iter()
            .map(|e| e.get_end(time_signature))
            .max()
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
    pub fn get_events_starting_between(&self, start: MusicTime, end: MusicTime, start_exclusive: bool) -> Vec<Event> {
        if (start_exclusive && start >= end) || start > end {
            return Vec::new()
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
            .for_each(|e|
                e.start = e.start.with(time_signature) + offset
            );
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
        Track {
            identifier: self.identifier,
            instrument: self.instrument,
            events,
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
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => panic!("Invalid note number")
        }.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Composition {
    pub tracks: Vec<Track>,
    pub time_signature: TimeSignature
}

impl Composition {
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
            "sine" => Ok(Instrument::SineWave),
            _ => Err(format!("Unknown instrument: {}", s))
        }
    }
}