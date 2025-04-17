use std::ops::DerefMut;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use crate::player::{AudioPlayer, Player};
use crate::scheduler::Scheduler;

pub fn run<S: DerefMut<Target=Scheduler> + Send>(scheduler: S, scheduler_tick_ms: u64, player: Player) {
    let (event_send, event_recv) = mpsc::channel();
    thread::scope(move |s| {
        s.spawn(move || {
            let start_time = SystemTime::now();
            let mut scheduler = scheduler;
            loop {
                if scheduler.ended() {
                    break;
                }
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

pub fn run_midi<S: DerefMut<Target=Scheduler> + Send, P: AudioPlayer>(scheduler: S, scheduler_tick_ms: u64, mut player: P) {
    let (event_send, event_recv) = mpsc::channel();
    thread::scope(move |s| {
        s.spawn(move || {
            let start_time = SystemTime::now();
            let mut scheduler = scheduler;
            loop {
                if scheduler.ended() {
                    break;
                }
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