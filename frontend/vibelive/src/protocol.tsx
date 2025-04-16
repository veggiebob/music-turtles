/*

Rust Struct Specifications

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
    Split(Vec<MusicString>),
    Repeat(usize, MusicString),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Symbol {
    NT(NonTerminal),
    T(Terminal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
    Note(Pitch),
    Rest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MetaControl {
    ChangeInstrument(Instrument),
    ChangeVolume(Volume),
}

#[derive(Serialize, Deserialize)]
pub struct TracedString {
    original: MusicString,
    productions: HashMap<usize, (Production, TracedString)>
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct MusicTime(pub Measure, pub Beat);

pub type BPM = f32;

pub type Measure = u32;


pub type BeatUnit = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Beat(Ratio<BeatUnit>);

/// Custom serialization for Beat
#[derive(Deserialize)]
struct Beat {
    numerator: u32,
    denominator: u32,
}
///

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TimeSignature(pub BeatUnit, pub BeatUnit);

pub type NoteNum = u8;
pub type Octave = i8;

pub type Frequency = f32;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Pitch(pub Octave, pub NoteNum);

*/

interface Grammar {
  start: NonTerminal;
  productions: Production[];
}

type Production = [NonTerminal, MusicString];

type MusicString = MusicPrimitive[];

type MusicPrimitive = Simple | Split | Repeat;
type Simple = { type: "Simple"; symbol: Symbol };
type Split = { type: "Split"; strings: MusicString[] };
type Repeat = { type: "Repeat"; times: number; string: MusicString };

type Symbol = NonTerminalSymbol | TerminalSymbol;
type NonTerminalSymbol = { type: "NT"; nt: NonTerminal };
type TerminalSymbol = { type: "T"; t: Terminal };

type NonTerminal = { type: "Custom"; Custom: string };

type Terminal = Music | Meta;
type Music = { type: "Music"; duration: MusicTime; note: TerminalNote };
type Meta = { type: "Meta"; control: MetaControl };

type TerminalNote = Note | Rest;
type Note = { type: "Note"; pitch: Pitch };
type Rest = { type: "Rest" };

type MetaControl = ChangeInstrument | ChangeVolume;
type ChangeInstrument = { type: "ChangeInstrument"; instrument: Instrument };
type ChangeVolume = { type: "ChangeVolume"; volume: Volume };

type Instrument = string;
type Volume = number;

type TracedString = {
  original: MusicString;
  productions: Map<number, [Production, TracedString]>;
};
type MusicTime = { measure: Measure; beat: Beat };
type BPM = number;
type Measure = number;
type BeatUnit = number;
type Beat = { numerator: number; denominator: number };
type TimeSignature = { numerator: BeatUnit; denominator: BeatUnit };
type NoteNum = number;
type Octave = number;
type Frequency = number;
type Pitch = [Octave, NoteNum];

export type {
  Simple,
  Split,
  Repeat,
  Symbol,
  NonTerminalSymbol,
  TerminalSymbol,
  NonTerminal,
  Terminal,
  Music,
  Meta,
  TerminalNote,
  Note,
  Rest,
  MetaControl,
  ChangeInstrument,
  ChangeVolume,
  Instrument,
  Volume,
  TracedString,
  MusicTime,
  BPM,
  Measure,
  BeatUnit,
  Beat,
  TimeSignature,
  NoteNum,
  Octave,
  Frequency,
  Pitch,
};
export type { MusicPrimitive, MusicString, Production, Grammar };