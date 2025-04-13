use std::ops::DerefMut;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use rodio::Source;
use crate::composition::{Event, Instrument, Pitch, Track, TrackId, Volume};
use crate::player::Player;
use crate::scheduler::Scheduler;
use crate::time::{Beat, MusicTime, TimeSignature};

mod player;
mod scheduler;
mod composition;
mod time;
mod cfg;

pub fn run<S: DerefMut<Target=Scheduler> + Send>(scheduler: S, scheduler_tick_ms: u64, player: Player) {
    let (event_send, event_recv) = mpsc::channel();
    thread::scope(move |s| {
        s.spawn(move || {
            let start_time = SystemTime::now();
            let mut scheduler = scheduler;
            loop {
                let elapsed_s = start_time.elapsed().unwrap().as_secs_f32();
                let sc = scheduler.deref_mut();
                let events = sc.get_next_events_and_update(elapsed_s);
                for event in events {
                    event_send.send(event).unwrap();
                }
                thread::sleep(Duration::from_millis(scheduler_tick_ms));
            }
        });
        player.play_from_ordered_channel(event_recv);
    });
}

fn main() {
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
