use crate::time::Beat;
use rodio::Source;
use std::ops::DerefMut;
#[macro_use] extern crate rocket;

mod player;
mod scheduler;
mod composition;

mod time;
mod cfg;
pub mod serde;

#[cfg(test)]
mod test;
mod local_playback;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}