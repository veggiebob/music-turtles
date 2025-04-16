use std::collections::HashMap;
use crate::composition::{Composition, Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::time::{MusicTime, TimeSignature};

/// Grammars that generate MusicStrings
pub struct GrammarProduction {
    // todo: all the productions codified so that they can be undone
}

pub struct MusicString(pub Vec<MusicPrimitive>);

pub enum MusicPrimitive {
    Simple(Symbol),
    Split(Vec<MusicString>),
    Repeat(usize, MusicString)
}

pub enum Symbol {
    NT(NonTerminal),
    T(Terminal)
}

pub enum NonTerminal {
    Custom(String)
}

pub enum Terminal {
    Music {
        duration: MusicTime,
        note: TerminalNote
    },
    Meta(MetaControl)
}

pub enum MetaControl {
    ChangeInstrument(Instrument),
    ChangeVolume(Volume)
}

pub enum TerminalNote {
    Note(Pitch),
    Rest
}

pub struct Grammar {
    start: Symbol,
    productions: Vec<(Symbol, MusicString)>
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
        let mut current_instrument = Instrument::SineWave;
        let mut current_volume = Volume(50);
        for mp in self.0.iter() {
            let duration = match mp {
                MusicPrimitive::Simple(sym) => {
                    match sym {
                        Symbol::NT(_) => {
                            MusicTime::zero()
                        }
                        Symbol::T(Terminal::Music {note, duration}) => {
                            match note {
                                TerminalNote::Note(pitch) => {
                                    add_event(&mut tracks, Event {
                                        start: current_mt,
                                        duration: duration.with(time_signature).total_beats(),
                                        volume: current_volume,
                                        pitch: *pitch,
                                    }, current_instrument);
                                    *duration
                                }
                                TerminalNote::Rest => {
                                    *duration
                                }
                            }
                        }
                        Symbol::T(Terminal::Meta(control)) => {
                            match control {
                                MetaControl::ChangeInstrument(i) => {
                                    current_instrument = *i;
                                }
                                MetaControl::ChangeVolume(v) => {
                                    current_volume = *v;
                                }
                            }
                            MusicTime::zero()
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
                    let mut offset = MusicTime::zero();
                    for _i in 0..*n {
                        let mut comp_i = composed.clone();
                        comp_i.shift_by(offset);
                        add_composition(&mut tracks, comp_i);
                        offset = offset.with(time_signature) + duration;
                    }
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