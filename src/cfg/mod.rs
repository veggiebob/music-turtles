use std::collections::HashMap;
use std::ops::Add;
use crate::composition::{Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::time::{MusicTime, TimeSignature};

pub enum NonTerminal {
    Custom(String)
}

pub enum Symbol {
    NT(NonTerminal),
    T(Terminal)
}

pub struct Terminal {
    duration: MusicTime,
    note: TerminalNote
}

pub enum TerminalNote {
    Note {
        instrument: Instrument,
        pitch: Pitch
    },
    Rest
}

pub enum MusicPrimitive {
    Simple(Symbol),
    Split(Vec<MusicString>),
    Repeat(usize, MusicString)
}

pub struct MusicString(Vec<MusicPrimitive>);

pub struct GrammarProduction {
    // todo: all the productions codified so that they can be undone
}

pub struct Grammar {
    start: Symbol,
    productions: Vec<(Symbol, MusicString)>
}

pub struct Composition {
    tracks: Vec<Track>,
    time_signature: TimeSignature
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
impl MusicString {
    pub fn compose(&self, time_signature: TimeSignature) -> Composition {
        let mut tracks = HashMap::new();
        fn add_event(tracks: &mut HashMap<Instrument, Track>, e: Event, instrument: Instrument) {
            if let Some(mut track) = tracks.get_mut(&instrument) {
                track.events.push(e);
            } else {
                tracks.insert(instrument, Track {
                    identifier: TrackId::Instrument(instrument),
                    instrument,
                    events: vec![e],
                });
            }
        }
        fn add_track(tracks: &mut HashMap<Instrument, Track>, track: Track) {
            if let Some(mtrack) = tracks.remove(&track.instrument) {
                tracks.insert(mtrack.instrument, mtrack + track);
            } else {
                tracks.insert(track.instrument, track);
            }
        }
        fn add_composition(tracks: &mut HashMap<Instrument, Track>, composition: Composition) {
            for track in composition.tracks {
                add_track(tracks, track);
            }
        }
        let mut current_mt = MusicTime::zero();
        for mp in self.0.iter() {
            let duration = match mp {
                MusicPrimitive::Simple(sym) => {
                    match sym {
                        Symbol::NT(_) => {
                            MusicTime::zero()
                        }
                        Symbol::T(Terminal {note, duration}) => {
                            match note {
                                TerminalNote::Note { pitch, instrument } => {
                                    add_event(&mut tracks, Event {
                                        start: current_mt,
                                        duration: duration.with(time_signature).total_beats(),
                                        volume: Volume(50), // idk
                                        pitch: *pitch,
                                    }, *instrument);
                                    *duration
                                }
                                TerminalNote::Rest => {
                                    *duration
                                }
                            }
                        }
                    }
                }
                MusicPrimitive::Split(mss) => {
                    let comps: Vec<_> = mss.into_iter()
                        .map(|ms| ms.compose(time_signature))
                        .map(|c| (c.get_duration(), c))
                        .collect();
                    let uniform_duration = match comps.first() {
                        Some((duration, _c)) => {
                            if comps.iter().all(|(d, _c)| d == duration) {
                                Some(*duration)
                            } else {
                                None
                            }
                        },
                        // there are none, so yes they are
                        None => None
                    };
                    if let Some(dur) = uniform_duration {
                        for (_d, comp) in comps {
                            add_composition(&mut tracks, comp);
                        }
                        dur
                    } else {
                        panic!("Not all split tracks have the same duration!!");
                    }
                }
                MusicPrimitive::Repeat(n, ms) => {
                    let composed = ms.compose(time_signature);
                    let duration = composed.get_duration();
                    // todo: repeat the actual tracks
                    add_composition(&mut tracks, composed);
                    let mut total_duration = MusicTime::zero();
                    for _i in 0..*n {
                        total_duration = total_duration.with(time_signature) + duration;
                    }
                    total_duration
                }
            };
            current_mt = current_mt.with(time_signature) + duration;
        }
        Composition {
            tracks: tracks.into_values().collect(),
            time_signature,
        }
    }
}