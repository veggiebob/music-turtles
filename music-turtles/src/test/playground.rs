use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use crate::cfg::{Grammar, MusicString};
use crate::composition::{Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::local_playback::{run, run_midi};
use crate::player::{MidiPlayer, Player};
use crate::scheduler::Scheduler;
use crate::time::{Beat, MusicTime, TimeSignature};

// ignore tests that play sounds
#[ignore]
#[test]
fn compose_something() {
    let input = "{[3][:c<2> :d<2>] | [3][:c :g :f# :g]}";
    // let input = "[2][:c :d :e {:e | :g}]";
    let string = MusicString::from_str(input).unwrap();
    let music = string.compose(TimeSignature::common(), None).unwrap();
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
    let player = MidiPlayer::new("test".to_string(), HashMap::new()).unwrap();
    thread::sleep(Duration::from_millis(1000)); // give player time to get ready
    // run(&mut scheduler, 50, player);
    run_midi(Arc::new(Mutex::new(scheduler)), 50, player);
}

// ignore tests that play sounds
#[ignore]
#[test]
fn run_file_grammar() {
    let input = "S";
    let mtx_path = "../data/stress-test1.mtx";
    let mtx_contents = std::fs::read_to_string(mtx_path).unwrap();
    let grammar = Grammar::from_str(&mtx_contents).unwrap();
    let mut string = MusicString::from_str(input).unwrap();
    for i in 0..4 {
        println!("After {} iters: {}", i, string.to_string());
        string = string.parallel_rewrite(&grammar, true, true);
    }
    println!("Final string: {}", string.to_string());

    let music = string.compose(TimeSignature::common(), None).unwrap();
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
    let player = MidiPlayer::new("test".to_string(), HashMap::new()).unwrap();
    thread::sleep(Duration::from_millis(1000)); // give player time to get ready
    run_midi(Arc::new(Mutex::new(scheduler)), 50, player);
}

// ignore tests that play sounds
#[ignore]
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
                rests: vec![],
            }, MusicTime(0, Beat::zero())),
        ],
        lookahead: MusicTime(1, Beat::zero()),
        looped: true,
        loop_time: MusicTime(1, Beat::zero()),
    };
    run(&mut scheduler, 50, player);
}