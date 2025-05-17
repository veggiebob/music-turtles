use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use crate::time::{Beat, MusicTime, Seconds, TimeSignature, BPM};
use rodio::Source;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use midir::{MidiOutput, MidiOutputConnection};
use midly::live::LiveEvent;
use midly::MidiMessage;
use rocket::http::Status;
use rocket::State;
use crate::cfg::{Grammar, MusicString};
use crate::cfg::scan::{consume, GrammarScanner, ScanError};
use crate::cfg::scan::Scanner;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};
use rocket_cors::CorsOptions;
use crate::cfg::interactive::TracedString;
use crate::composition::Instrument;
use crate::composition::Instrument::*;
use crate::local_playback::{run, run_midi};
use crate::player::{MidiPlayer, Player};
use crate::scheduler::Scheduler;
use simplelog::*;

#[macro_use]
extern crate log;
extern crate simplelog;

extern crate rocket;

mod player;
mod scheduler;
mod composition;

mod time;
mod cfg;
pub mod serde;

#[cfg(test)]
mod test;
pub mod local_playback;
mod constants;

pub struct ServerConfig {
    pub data_path: String,
}

// #[get("/grammar/<filename>")]
// async fn grammar(filename: &str, config: &State<ServerConfig>) -> Result<Json<Grammar>, Status> {
//     // concatenate config path with filename and read contents
//     // using path join
//     let path = std::path::Path::new(&config.data_path).join(filename);
//     let contents = std::fs::read_to_string(path)
//         .map_err(|_| Status::NotFound)?;
//     let contents = contents.trim();
//     let (gram, _empty) = GrammarScanner.scan(&contents)
//         .map_err(|e| {
//             eprintln!("Error parsing grammar: {:?}", e);
//             Status::InternalServerError
//         })?;
//     Ok(Json(gram))
// }

// #[post("/play", format = "json", data = "<music_tree>")]
// async fn play(music_tree: Json<TracedString>) -> Result<(), Status> {
//     let music_string = music_tree.into_inner().render();
//     let time_sig = TimeSignature::common();
//     let composition = music_string.compose(time_sig, None).unwrap();
//     let mut scheduler = Scheduler {
//         bpm: 80.0,
//         time_signature: time_sig,
//         tracks: vec![],
//         lookahead: MusicTime::measures(1),
//         looped: false,
//         loop_time: MusicTime::zero(),
//     };
//     scheduler.set_composition(composition);
//     let player = Player::new();
//     run(
//         &mut scheduler,
//         50,
//         player,
//     );
//     Ok(())
// }

// #[launch]
// fn rocket() -> _ {
//     let cors = CorsOptions::default()
//         .to_cors()
//         .expect("error creating CORS fairing");
//     rocket::build()
//         .attach(cors)
//         .manage(ServerConfig {
//             data_path: "../data".to_string()
//         })
//         .mount("/", routes![grammar, play])
// }

fn file_watcher<F>(file: &str, mut f: F, period: Seconds) -> JoinHandle<()>
where
    F: FnMut(String) + Send + 'static,
{
    let filename = file.to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs_f32(period));
        let filename = filename;
        let mut last_contents = String::new();
        loop {
            info!("Reading file...");
            if let Ok(contents) = std::fs::read_to_string(&filename) {
                if last_contents != contents {
                    f(contents.clone());
                    info!("reloaded");
                    last_contents = contents;
                } else {
                    info!("File is unchanged.");
                }
            } else {
                info!("Warning: failed to update from file {filename}");
            }
            thread::sleep(Duration::from_secs_f32(period));
        }
    })
}

pub fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
    ]).unwrap();
    let axiom = "S";
    let time_signature = TimeSignature::common();
    let bpm: BPM = 120.0;
    let grm_path = "../data/beat-1.grm";
    let grm_contents = std::fs::read_to_string(grm_path).unwrap();
    let grammar = Grammar::from_str(&grm_contents).unwrap();
    let mut string = MusicString::from_str(axiom).unwrap();
    for _i in 0..20 {
        // println!("After {} iters: {}", i, string.to_string());
        string = string.parallel_rewrite(&grammar, true);
    }
    info!("Final string: {}", string.to_string());

    let music = string.compose(time_signature, None).unwrap();
    info!("Final music: \n{}", music.visualize(150));
    println!("{music:#?}");
    let mut scheduler = Scheduler {
        bpm,
        time_signature,
        tracks: vec![],
        lookahead: MusicTime::measures(1),
        looped: true,
        loop_time: music.get_duration(),
    };
    let channel_mapping = Instrument::values().into_iter().map(|i| (i, match i {
        BongoHigh => 1,
        BongoLow => 2,
        Shaker1 => 3,
        Shaker2 => 4,
        _ => 0xf,
    })).collect();
    scheduler.set_composition(music);
    let sched = Arc::new(Mutex::new(scheduler));
    let update_sched = Arc::clone(&sched);
    let watcher = file_watcher(grm_path, move |file_contents| {
        match Grammar::from_str(&file_contents) {
            Ok(g) => {
                match MusicString::from_str(axiom) {
                    Ok(s) => {
                        let string = s.parallel_rewrite_n(&g, true, 20);
                        info!("Final string: {string:?}");
                        match string.compose(time_signature, None) {
                            Ok(comp) => {
                                info!("unlocking...");
                                let mut lock = update_sched.lock().unwrap();
                                lock.set_composition(comp);
                                info!("Reloaded composition");
                            }
                            Err(e) => warn!("Couldn't compose.\n{e:?}")
                        }
                    }
                    Err(e) => warn!("Couldn't parse axiom: {e:?}")
                }
            }
            Err(e) => warn!("Unable to parse grammar: {e:?}")
        }
    }, 2.);

    let player = MidiPlayer::new("music-turtles".to_string(), channel_mapping).unwrap();
    thread::sleep(Duration::from_millis(1000)); // give player time to get ready
    run_midi(sched, 100, player);
}

pub fn other() -> Result<(), Box<dyn std::error::Error>> {
    let midi_out = MidiOutput::new("test").unwrap();
    // List available ports
    let out_ports = midi_out.ports();
    println!("Available output ports:");
    for (i, p) in out_ports.iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }

    // Pick a port
    print!("Select output port: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let index: usize = input.trim().parse()?;
    let port = &out_ports[index];

    // Connect
    let mut conn: MidiOutputConnection = midi_out.connect(port, "midir-connection")?;

    fn note_on_message(channel: u8, key: u8) -> Vec<u8> {
        let ev = LiveEvent::Midi {
            channel: channel.into(),
            message: MidiMessage::NoteOn {
                key: key.into(),
                vel: 127.into(),
            },
        };
        let mut buf = Vec::new();
        ev.write(&mut buf).unwrap();
        buf
    }

    fn note_off_message(channel: u8, key: u8) -> Vec<u8> {
        let ev = LiveEvent::Midi {
            channel: channel.into(),
            message: MidiMessage::NoteOff {
                key: key.into(),
                vel: 0.into(), // standard to send 0 velocity
            },
        };
        let mut buf = Vec::new();
        ev.write(&mut buf).unwrap();
        buf
    }

    loop {
        // Send Note On
        let note = 60; // Middle C
        let channel = 0;
        conn.send(&note_on_message(channel, note))?;
        println!("Note on");

        thread::sleep(Duration::from_secs(1));

        // Send Note Off
        conn.send(&note_off_message(channel, note))?;
        println!("Note off");

        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}