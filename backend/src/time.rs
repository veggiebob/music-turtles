use std::fmt::Display;
use std::ops::{Add, Mul, Sub};
use num::rational::Ratio;
use num::{FromPrimitive, ToPrimitive, Zero};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;

pub type Seconds = f32;

/// Represents either a duration or absolute position in music.
/// The measures must be positive. The beats should also be positive, and constrained
/// within the measure, if it is an absolute position.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct MusicTime(pub Measure, pub Beat);

pub type BPM = f32;

pub type Measure = u32;


pub type BeatUnit = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Beat(Ratio<BeatUnit>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TimeSignature(pub BeatUnit, pub BeatUnit);

#[derive(Debug, Clone, Copy)]
pub struct TimeCompression(pub Ratio<isize>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MusicTimeWithSignature {
    pub time: MusicTime,
    pub time_signature: TimeSignature,
}

impl Beat {
    pub fn new(num: BeatUnit, denom: BeatUnit) -> Self {
        Beat(Ratio::new(num, denom))
    }

    pub fn whole(num: BeatUnit) -> Self {
        Beat(Ratio::new(num, 1))
    }

    pub fn as_float(&self) -> f32 {
        self.0.to_f32().unwrap_or_else(|| {
            println!("WARNING: Beat {self:?} could not be converted to f32. Defaulting to 0.");
            0.
        })
    }

    pub fn as_music_time(&self, time_signature: TimeSignature) -> MusicTime {
        let measures = (self.0 / time_signature.0).floor().to_integer();
        let leftover = self.0 % time_signature.0;
        MusicTime(measures, Beat(leftover))
    }

    pub fn zero() -> Self {
        Beat(Ratio::zero())
    }

    pub fn numerator(&self) -> BeatUnit {
        self.0.numer().to_u32().unwrap_or_else(|| {
            println!("WARNING: Beat {self:?} numerator could not be converted to u32. Defaulting to 0.");
            0
        })
    }

    pub fn denominator(&self) -> BeatUnit {
        self.0.denom().to_u32().unwrap_or_else(|| {
            println!("WARNING: Beat {self:?} denominator could not be converted to u32. Defaulting to 1.");
            1
        })
    }
}

impl MusicTime {
    pub fn with(self, time_signature: TimeSignature) -> MusicTimeWithSignature {
        MusicTimeWithSignature {
            time_signature,
            time: self
        }
    }

    pub fn from_seconds(time_signature: TimeSignature, bpm: BPM, seconds: Seconds) -> Self {
        let bps = bpm / 60.;
        let beats = bps * seconds;
        // instead of using Ratio::from_f32, I'll calculate the fraction myself
        let precision = 1000000.0; // to avoid floating point precision issues
        let numerator = (beats * precision).floor() as BeatUnit;
        let denominator = precision as BeatUnit;
        let beats = Beat(Ratio::new(numerator, denominator));
        beats.as_music_time(time_signature)
    }

    pub fn from_whole_beats(time_signature: TimeSignature, beats: BeatUnit) -> Self {
        let measures = beats / time_signature.0;
        let beats = beats % time_signature.0;
        MusicTime(measures, Beat::whole(beats))
    }

    pub fn to_seconds(&self, time_signature: TimeSignature, bpm: BPM) -> Seconds {
        let MusicTime(measures, beats) = *self;
        let total_beats = (measures * time_signature.0) as f32 + beats.as_float();
        total_beats * 60. / bpm
    }

    pub fn zero() -> Self {
        MusicTime(0, Beat::zero())
    }

    pub fn beats(beats: BeatUnit) -> Self {
        MusicTime(0, Beat::whole(beats))
    }

    pub fn measures(measures: Measure) -> Self {
        MusicTime(measures, Beat::zero())
    }
}

impl Add<Beat> for Beat {
    type Output = Beat;

    fn add(self, rhs: Beat) -> Self::Output {
        Beat(self.0 + rhs.0)
    }
}

impl Sub<Beat> for Beat {
    type Output = Beat;

    fn sub(self, rhs: Beat) -> Self::Output {
        Beat(self.0 - rhs.0)
    }
}

impl Add<MusicTime> for MusicTimeWithSignature {
    type Output = MusicTime;

    fn add(self, rhs: MusicTime) -> Self::Output {
        let MusicTime(measure, beat) = self.time;
        let MusicTime(measure2, beat2) = rhs;
        let new_measure = measure + measure2;
        let MusicTime(beat_measures, beat) = (beat + beat2).as_music_time(self.time_signature);
        MusicTime(new_measure + beat_measures, beat)
    }
}

impl Sub<MusicTime> for MusicTimeWithSignature {
    type Output = MusicTime;

    fn sub(self, rhs: MusicTime) -> Self::Output {
        let MusicTime(measure, beat) = self.time;
        let MusicTime(measure2, beat2) = rhs;
        let mut new_measure = measure - measure2;
        let mut new_beat = beat;
        if beat2 > new_beat {
            new_beat = new_beat + Beat::whole(self.time_signature.0);
            new_measure -= 1;
        }
        new_beat = new_beat - beat2;
        let MusicTime(beat_measures, beats) = new_beat.as_music_time(self.time_signature);
        MusicTime(beat_measures + new_measure, beats)
    }
}

impl Mul<Ratio<BeatUnit>> for MusicTimeWithSignature {
    type Output = MusicTimeWithSignature;

    fn mul(self, rhs: Ratio<BeatUnit>) -> Self::Output {
        let total_beats = self.total_beats();
        let new_total_beats = Beat(total_beats.0 * rhs);
        let music_time = new_total_beats.as_music_time(self.time_signature);
        MusicTimeWithSignature {
            time: music_time,
            time_signature: self.time_signature,
        }
    }
}

impl MusicTimeWithSignature {
    pub fn total_beats(&self) -> Beat {
        Beat::new(self.time.0 * self.time_signature.0 as BeatUnit, 1) + self.time.1
    }
}

impl TimeSignature {
    pub fn common() -> Self {
        TimeSignature(4, 4)
    }
}

impl Serialize for Beat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("Beat", 2)?;
        let num = self.numerator();
        let denom = self.denominator();
        state.serialize_field("numerator", &num)?;
        state.serialize_field("denominator", &denom)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Beat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct Beat {
            numerator: u32,
            denominator: u32,
        }

        let data = Beat::deserialize(deserializer)?;
        Ok(crate::Beat::new(data.numerator, data.denominator))
    }
}

impl Serialize for TimeCompression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("TimeCompression", 1)?;
        state.serialize_field("numerator", &self.0.numer())?;
        state.serialize_field("denominator", &self.0.denom())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TimeCompression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct TimeCompression {
            numerator: isize,
            denominator: isize,
        }

        let data = TimeCompression::deserialize(deserializer)?;
        Ok(TimeCompression(Ratio::new(data.numerator, data.denominator)))
    }
}

impl Display for TimeCompression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0.numer(), self.0.denom())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_music_time() {
        let mt1 = MusicTime(0, Beat::new(0, 1));
        let mt2 = MusicTime(0, Beat::new(1191, 23819));
        assert!(mt1 < mt2);
        assert!(mt2 > mt1);
    }

    #[test]
    fn test_music_time_sub_1() {
        let ts = TimeSignature::common();
        let mt1 = MusicTime(1, Beat::whole(0));
        let mt2 = MusicTime(0, Beat::whole(3));
        assert_eq!(mt1.with(ts) - mt2, MusicTime(0, Beat::whole(1)));
    }

    #[test]
    fn test_music_time_sub_2() {
        let ts = TimeSignature::common();
        let mt1 = MusicTime(1, Beat::whole(3));
        let mt2 = MusicTime(0, Beat::whole(0));
        assert_eq!(mt1.with(ts) - mt2, MusicTime(1, Beat::whole(3)));
    }

    #[test]
    fn test_music_time_sub_3() {
        let ts = TimeSignature::common();
        let mt1 = MusicTime(2, Beat::whole(0));
        let mt2 = MusicTime(0, Beat::whole(3));
        assert_eq!(mt1.with(ts) - mt2, MusicTime(1, Beat::whole(1)));
    }
}