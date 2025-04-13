use std::ops::Add;
use crate::time::{Beat, MusicTime, TimeSignature};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd)]
pub enum Instrument {
    SineWave
}

/// [0, 12)
pub type NoteNum = u8;
pub type Octave = i8;

pub type Frequency = f32;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
}

