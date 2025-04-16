use std::ops::{Add, Sub};
use num::rational::Ratio;
use num::{FromPrimitive, ToPrimitive, Zero};

pub type Seconds = f32;

/// Represents either a duration or absolute position in music.
/// The measures must be positive. The beats should also be positive, and constrained
/// within the measure, if it is an absolute position.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MusicTime(pub Measure, pub Beat);

pub type BPM = f32;

pub type Measure = u32;


pub type BeatUnit = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Beat(Ratio<BeatUnit>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TimeSignature(pub BeatUnit, pub BeatUnit);

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
        let beats = Beat(Ratio::from_f32(beats).unwrap());
        beats.as_music_time(time_signature)
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
        let mut new_beat = beat - beat2;
        while new_beat.0 < Ratio::zero() {
            new_measure -= 1;
            new_beat.0 += self.time_signature.0; // add number of beats in a measure
        }
        let MusicTime(beat_measures, beats) = new_beat.as_music_time(self.time_signature);
        MusicTime(beat_measures + new_measure, beats)
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

}