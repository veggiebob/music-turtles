use crate::time::{Beat, MusicTime};

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

impl Track {
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
            .map(|e| {
                println!("event: {e:?}");
                e
            })
            .map(|e| *e)
            .collect::<Vec<_>>();
        es.sort();
        es
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

