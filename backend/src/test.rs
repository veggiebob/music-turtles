use std::str::FromStr;
use std::thread;
use std::time::Duration;
use crate::cfg::{Grammar, MusicString};
use crate::composition::{Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::player::{MidiPlayer, Player};
use crate::local_playback::{run, run_midi};
use crate::scheduler::Scheduler;
use crate::time::{Beat, MusicTime, TimeSignature};

#[test]
fn compose_something() {
    let input = "{[3][:c<2> :d<2>] | [3][:c :g :f# :g]}";
    // let input = "[2][:c :d :e {:e | :g}]";
    let string = MusicString::from_str(input).unwrap();
    let music = string.compose(TimeSignature::common(), None);
    println!("{music:#?}");
    let mut scheduler = Scheduler {
        bpm: 80.0,
        time_signature: TimeSignature(4, 4),
        tracks: vec![],
        lookahead: MusicTime::measures(1),
        looped: false,
        loop_time: MusicTime::measures(1),
    };
    scheduler.set_composition(music);
    let player = MidiPlayer::new("test".to_string()).unwrap();
    thread::sleep(Duration::from_millis(1000)); // give player time to get ready
    // run(&mut scheduler, 50, player);
    run_midi(&mut scheduler, 50, player);
}

#[test]
fn run_file_grammar() {
    let input = "S";
    let grm_path = "../data/grm3.grm";
    let grm_contents = std::fs::read_to_string(grm_path).unwrap();
    let grammar = Grammar::from_str(&grm_contents).unwrap();
    let mut string = MusicString::from_str(input).unwrap();
    for i in 0..4 {
        println!("After {} iters: {}", i, string.to_string());
        string = string.parallel_rewrite(&grammar, true);
    }
    println!("Final string: {}", string.to_string());

    let music = string.compose(TimeSignature::common(), None);
    // println!("{music:#?}");
    let mut scheduler = Scheduler {
        bpm: 80.0,
        time_signature: TimeSignature(4, 4),
        tracks: vec![],
        lookahead: MusicTime::measures(1),
        looped: false,
        loop_time: MusicTime::measures(1),
    };
    scheduler.set_composition(music);
    let player = MidiPlayer::new("test".to_string()).unwrap();
    thread::sleep(Duration::from_millis(1000)); // give player time to get ready
    run_midi(&mut scheduler, 50, player);
}

#[test]
fn a() {
    let player = Player::new();
    let mut scheduler = Scheduler {
        bpm: 80.0,
        time_signature: TimeSignature(4, 4),
        tracks: vec![
            (Track {
                identifier: TrackId::Custom(0),
                instrument: Instrument::SineWave,
                events: vec![
                    Event {
                        start: MusicTime(0, Beat::zero()),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 0),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(1, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 2),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(2, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 4),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(3, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 5),
                    },
                    Event {
                        start: MusicTime(0, Beat::zero()),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 4),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(1, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 5),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(2, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 7),
                    },
                    Event {
                        start: MusicTime(0, Beat::new(3, 1)),
                        duration: Beat::new(1, 1),
                        volume: Volume(20),
                        pitch: Pitch(4, 9),
                    }
                ],
            }, MusicTime(0, Beat::zero())),
        ],
        lookahead: MusicTime(1, Beat::zero()),
        looped: true,
        loop_time: MusicTime(1, Beat::zero()),
    };
    run(&mut scheduler, 50, player);
}