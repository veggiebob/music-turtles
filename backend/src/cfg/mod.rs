pub mod scan;
pub mod interactive;

use crate::cfg::scan::{consume, MusicStringScanner, ScanError};
use crate::cfg::scan::{GrammarScanner, Scanner};
use crate::composition::{Composition, Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::time::{Beat, MusicTime, TimeSignature};
use num::Zero;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    start: NonTerminal,
    productions: Vec<Production>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Production(NonTerminal, MusicString);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicString(pub Vec<MusicPrimitive>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MusicPrimitive {
    Simple(Symbol),
    Split {
        branches: Vec<MusicString>
    },
    Repeat {
        num: usize,
        content: MusicString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Symbol {
    NT(NonTerminal),
    T(Terminal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NonTerminal {
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Terminal {
    Music {
        duration: MusicTime,
        note: TerminalNote,
    },
    Meta(MetaControl),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TerminalNote {
    Note {
        pitch: Pitch
    },
    Rest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MetaControl {
    ChangeInstrument(Instrument),
    ChangeVolume(Volume),
}

impl Grammar {
    pub fn new(start: NonTerminal, productions: Vec<Production>) -> Self {
        Grammar { start, productions }
    }

    pub fn get_production(&self, nt: &NonTerminal) -> Option<&Production> {
        self.productions.iter().find(|p| &p.0 == nt)
    }

    pub fn get_production_random(
        &self,
        nt: &NonTerminal,
    ) -> Option<&Production> {
        let mut rng = rand::thread_rng();
        let productions: Vec<_> = self.productions.iter().filter(|p| &p.0 == nt).collect();
        if productions.is_empty() {
            None
        } else {
            Some(productions[rng.gen_range(0..productions.len())])
        }
    }
}

impl FromStr for Grammar {
    type Err = ScanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scanner = consume(GrammarScanner);
        let (grammar, _s) = scanner.scan(s)?;
        Ok(grammar)
    }
}

#[derive(Debug)]
pub enum ComposeError {
    MismatchedLengths(String),

}

impl MusicString {
    pub fn compose(&self, time_signature: TimeSignature, starting_instrument: Option<Instrument>) -> Result<Composition, ComposeError> {
        let mut tracks = HashMap::new();
        fn add_event(tracks: &mut HashMap<Instrument, Track>, e: Event, instrument: Instrument) {
            if let Some(mut track) = tracks.get_mut(&instrument) {
                track.events.push(e);
            } else {
                tracks.insert(
                    instrument,
                    Track {
                        identifier: TrackId::Instrument(instrument),
                        instrument,
                        events: vec![e],
                        rests: vec![],
                    },
                );
            }
        }

        fn add_rest_event(tracks: &mut HashMap<Instrument, Track>, e: Event, instrument: Instrument) {
            if let Some(mut track) = tracks.get_mut(&instrument) {
                track.rests.push(e);
            } else {
                tracks.insert(
                    instrument,
                    Track {
                        identifier: TrackId::Instrument(instrument),
                        instrument,
                        events: vec![],
                        rests: vec![e],
                    },
                );
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
        let mut current_instrument = starting_instrument.unwrap_or(Instrument::SineWave);
        let mut current_volume = Volume(50);
        for mp in self.0.iter() {
            let duration = match mp {
                MusicPrimitive::Simple(sym) => match sym {
                    Symbol::NT(_) => MusicTime::zero(),
                    Symbol::T(Terminal::Music { note, duration }) => match note {
                        TerminalNote::Note { pitch } => {
                            add_event(
                                &mut tracks,
                                Event {
                                    start: current_mt,
                                    duration: duration.with(time_signature).total_beats(),
                                    volume: current_volume,
                                    pitch: *pitch,
                                },
                                current_instrument,
                            );
                            *duration
                        }
                        TerminalNote::Rest => {
                            add_rest_event(
                                &mut tracks,
                                Event {
                                    start: current_mt,
                                    duration: duration.with(time_signature).total_beats(),
                                    volume: Volume(0),
                                    pitch: Pitch(0, 0),
                                },
                                current_instrument,
                            );
                            *duration
                        }
                    },
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
                },
                MusicPrimitive::Split { branches } => {
                    let comps: Vec<_> = branches
                        .into_iter()
                        .map(|ms| ms.compose(time_signature, Some(current_instrument)))
                        .err_first()?
                        .map(|mut c| {
                            c.shift_by(current_mt);
                            c
                        })
                        .map(|c| (c.get_duration(), c))
                        .collect();
                    let uniform_duration = match comps.first() {
                        Some((duration, _c)) => {
                            if comps.iter().all(|(d, _c)| d == duration) {
                                Some(*duration)
                            } else {
                                None
                            }
                        }
                        // there are none, so yes they are
                        None => Some(MusicTime::zero()),
                    };
                    if let Some(dur) = uniform_duration {
                        for (_d, comp) in comps {
                            add_composition(&mut tracks, comp);
                        }
                        dur
                    } else {
                        return Err(ComposeError::MismatchedLengths(
                            format!("Not all split tracks have the same duration: {:?}",
                                    comps.iter().map(|(d, c)| d).collect::<Vec<_>>()
                            )));
                    }
                }
                MusicPrimitive::Repeat { content, num } => {
                    let composed = content.compose(time_signature, Some(current_instrument))?;
                    let duration = composed.get_duration();
                    let mut offset = current_mt;
                    for _i in 0..*num {
                        let mut comp_i = composed.clone();
                        comp_i.shift_by(offset);
                        add_composition(&mut tracks, comp_i);
                        offset = offset.with(time_signature) + duration;
                    }
                    let mut total_duration = MusicTime::zero();
                    for _i in 0..*num {
                        total_duration = total_duration.with(time_signature) + duration;
                    }
                    // println!("total duration for {num} repeats is {total_duration:?}, or {:?} * {num}",
                    //          composed.get_duration());
                    total_duration
                }
            };
            current_mt = current_mt.with(time_signature) + duration;
        }
        Ok(Composition {
            tracks: tracks.into_values().collect(),
            time_signature,
        })
    }

    pub fn parallel_rewrite(&self, grammar: &Grammar, random: bool) -> Self {
        let mut new_string = vec![];
        for (i, mp) in self.0.iter().enumerate() {
            match mp {
                MusicPrimitive::Simple(x) => match x {
                    Symbol::NT(nt) => {
                        if let Some(Production(nt, ms)) = if random { grammar.get_production_random(nt) } else { grammar.get_production(nt) } {
                            new_string.extend(ms.clone().0);
                        } else {
                            println!("Warning: no production for {nt:?}");
                        }
                    }
                    x => {
                        new_string.push(MusicPrimitive::Simple(x.clone()));
                    }
                }
                MusicPrimitive::Split { branches } => {
                    let new_branches = branches
                        .iter()
                        .map(|ms| ms.parallel_rewrite(grammar, random))
                        .collect::<Vec<_>>();
                    new_string.push(MusicPrimitive::Split { branches: new_branches });
                }
                MusicPrimitive::Repeat { num, content } => {
                    let new_content = content.parallel_rewrite(grammar, random);
                    new_string.push(MusicPrimitive::Repeat {
                        num: *num,
                        content: new_content,
                    });
                }
            }
        }
        MusicString(new_string)
    }

    pub fn parallel_rewrite_n(&self, grammar: &Grammar, random: bool, n: usize) -> Self {
        let mut new_string = self.clone();
        for _i in 0..n {
            new_string = new_string.parallel_rewrite(grammar, random);
        }
        new_string
    }
}

impl ToString for MusicString {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for mp in &self.0 {
            match mp {
                MusicPrimitive::Simple(sym) => {
                    let sym_to_string = sym.to_string();
                    s.push_str(&sym_to_string);
                }
                MusicPrimitive::Split { branches } => {
                    s.push_str("{");
                    let str = branches.into_iter()
                        .map(|b| b.to_string())
                        .reduce(|b1, b2| b1 + " | " + &b2)
                        .unwrap_or("".to_string());
                    s.push_str(&str);
                    s.push('}');
                }
                MusicPrimitive::Repeat { num, content } => {
                    s.push_str(&format!("[{}][", num));
                    s.push_str(&content.to_string());
                    s.push(']');
                }
            }
            s.push(' ');
        }
        s
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match self {
            Symbol::NT(nt) => nt.to_string(),
            Symbol::T(t) => t.to_string(),
        }
    }
}

impl ToString for NonTerminal {
    fn to_string(&self) -> String {
        match self {
            NonTerminal::Custom(s) => s.clone(),
        }
    }
}

impl ToString for Terminal {
    fn to_string(&self) -> String {
        match self {
            Terminal::Music { duration, note } => {
                match note {
                    TerminalNote::Note { pitch } => {
                        let letter = pitch.letter_name();
                        format!(":{letter}<{}>", duration.to_string())
                    }
                    TerminalNote::Rest => {
                        format!(":_<{}>", duration.to_string())
                    }
                }
            }
            Terminal::Meta(control) => control.to_string(),
        }
    }
}

impl ToString for MusicTime {
    fn to_string(&self) -> String {
        let MusicTime(measures, beats) = self;
        let beat_str = if *beats == Beat::zero() {
            "0".to_string()
        } else {
            if beats.denominator() == 1 {
                format!("{}", beats.numerator())
            } else {
                format!("{}/{}", beats.numerator(), beats.denominator())
            }
        };
        if *measures == 0 {
            format!("{}", beat_str)
        } else {
            format!("{}m+{}", measures, beat_str)
        }
    }
}

impl ToString for MetaControl {
    fn to_string(&self) -> String {
        match self {
            MetaControl::ChangeInstrument(i) => format!("::i={:?}", i),
            MetaControl::ChangeVolume(v) => format!("::v={:?}", v),
        }
    }
}
impl FromStr for MusicString {
    type Err = ScanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scanner = consume(MusicStringScanner);
        scanner.scan(s).map(|(r, _s)| r)
    }
}

pub trait ReduceResultIter<I, E> {
    fn err_first(self) -> Result<impl Iterator<Item=I>, E>;
}

impl<I, E, T> ReduceResultIter<I, E> for T
where
    T: Iterator<Item=Result<I, E>>,
{
    fn err_first(self) -> Result<impl Iterator<Item=I>, E> {
        let mut processed = vec![];
        for e in self {
            match e {
                Ok(i) => processed.push(i),
                Err(e) => return Err(e)
            }
        }
        Ok(processed.into_iter())
    }
}


#[cfg(test)]
mod test {
    use std::io::Cursor;
    use serde_json::Deserializer;

    #[test]
    fn test() {
        let data = vec![1, 2, 3];
        let mut c = Cursor::new(data);
        let deserializer = Deserializer::new(c);
    }
}