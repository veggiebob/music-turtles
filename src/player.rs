use std::cmp::{max, min};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::SystemTime;
use rodio::{OutputStream, OutputStreamHandle, Source};
use crate::time::Seconds;

pub struct Player {
    stream: OutputStream,
    output_stream: OutputStreamHandle
}

pub trait Playable {
    /// get start time, duration, and actual sound
    fn get_source(&self) -> (Seconds, Seconds, Box<dyn Source<Item=f32> + Send + 'static>);
}

impl Player {
    pub fn new() -> Self {
        let (stream, output_stream) = OutputStream::try_default().unwrap();
        Player { stream, output_stream }
    }
    pub fn play(&self, source: impl Source<Item=f32> + Send + 'static) {
        let sink = rodio::Sink::try_new(&self.output_stream).unwrap();
        // thread::spawn(move || {
        //     let source: Box<dyn Source<Item=f32> + Send> = Box::new(source);
        //     sink.append(source);
        //     sink.sleep_until_end();
        // });
        let source: Box<dyn Source<Item=f32> + Send> = Box::new(source);
        sink.append(source);
        sink.detach();
    }

    /// Incoming events MUST BE IN ORDER
    pub fn play_from_ordered_channel<T: Playable>(&self, queue: Receiver<T>) {
        let start_pause = 0.1; // seconds
        let start_time = SystemTime::now() - std::time::Duration::from_secs_f32(start_pause);
        let mut end = start_time;
        for event in queue {
            let (start, duration, source) = event.get_source();
            let current_time = SystemTime::now();
            let elapsed = current_time.duration_since(start_time).unwrap().as_secs_f32();
            let wait_time = start - elapsed;
            // println!("Waiting for {wait_time} until {start}... (sound is {duration}s long)");
            if wait_time > 0. {
                thread::sleep(std::time::Duration::from_secs_f32(wait_time));
            }
            end = SystemTime::max(end, current_time + std::time::Duration::from_secs_f32(f32::max(wait_time, 0.) + duration));
            self.play(source);
        }
        // wait for the last sound to finish
        let wait_time = end.duration_since(SystemTime::now()).unwrap().as_secs_f32();
        if wait_time > 0. {
            std::thread::sleep(std::time::Duration::from_secs_f32(wait_time));
        }
    }
}