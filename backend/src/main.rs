use crate::time::{Beat, MusicTime, TimeSignature};
use rodio::Source;
use std::ops::DerefMut;
use rocket::http::Status;
use rocket::State;
use crate::cfg::Grammar;
use crate::cfg::scan::{consume, GrammarScanner};
use crate::cfg::scan::Scanner;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};
use rocket_cors::CorsOptions;
use crate::cfg::interactive::TracedString;
use crate::local_playback::run;
use crate::player::Player;
use crate::scheduler::Scheduler;

#[macro_use]
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

pub struct ServerConfig {
    pub data_path: String,
}

#[get("/grammar/<filename>")]
async fn grammar(filename: &str, config: &State<ServerConfig>) -> Result<Json<Grammar>, Status> {
    // concatenate config path with filename and read contents
    // using path join
    let path = std::path::Path::new(&config.data_path).join(filename);
    let contents = std::fs::read_to_string(path)
        .map_err(|_| Status::NotFound)?;
    let contents = contents.trim();
    let (gram, _empty) = GrammarScanner.scan(&contents)
        .map_err(|e| {
            eprintln!("Error parsing grammar: {:?}", e);
            Status::InternalServerError
        })?;
    Ok(Json(gram))
}

#[post("/play", format = "json", data = "<music_tree>")]
async fn play(music_tree: Json<TracedString>) -> Result<(), Status> {
    let music_string = music_tree.into_inner().render();
    let time_sig = TimeSignature::common();
    let composition = music_string.compose(time_sig);
    let mut scheduler = Scheduler {
        bpm: 80.0,
        time_signature: time_sig,
        tracks: vec![],
        lookahead: MusicTime::measures(1),
        looped: false,
        loop_time: MusicTime::zero(),
    };
    scheduler.set_composition(composition);
    let player = Player::new();
    run(
        &mut scheduler,
        50,
        player,
    );
    Ok(())
}

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .to_cors()
        .expect("error creating CORS fairing");
    rocket::build()
        .attach(cors)
        .manage(ServerConfig {
            data_path: "../data".to_string()
        })
        .mount("/", routes![grammar, play])
}