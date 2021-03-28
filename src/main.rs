extern crate clokwerk;
extern crate tokio;
extern crate warp;
extern crate uuid;

use clokwerk::{Scheduler, TimeUnits};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use warp::{Filter, http::Response};
use uuid::Uuid;
use std::fs;

fn base_camera_command() -> std::process::Command {
    let mut command = Command::new("raspistill");
    command.arg("-vf");
    command.arg("-hf");
    command.arg("-n");
    return command;
}

fn take_and_save_image(filepath: String) {
    let mut command = base_camera_command();
    command.arg("-o");
    command.arg(filepath);

    let mut child = command.spawn().expect("Command failed to start");
    let _ = child.wait().unwrap();
}

fn gen_timelapse_filename() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

fn take_timelapse_photo() {
    let filename = gen_timelapse_filename();
    let filepath = format!("images/{}.jpg", filename);

    take_and_save_image(filepath);
}

fn timelapse_thread() {
    let mut scheduler = Scheduler::new();

    scheduler.every(1.hour()).run(|| take_timelapse_photo());

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(500));
    }
}

fn take_instant_photo() -> Vec<u8> {
    let filename = Uuid::new_v4().to_hyphenated().to_string();
    let filepath = format!("/tmp/{}.jpg", filename);

    take_and_save_image(filepath.clone());

    let image = fs::read(filepath).unwrap();
    return image;
}

#[tokio::main]
async fn main() {
    let timelapse_child = thread::spawn(timelapse_thread);

    let index = warp::get().and(warp::path::end()).map(|| {
        let image = take_instant_photo();
        return Response::builder()
            .header("Content-Type", "image/jpeg")
            .body(image);
    });
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let routes = index.or(hello);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    timelapse_child.join().unwrap();
}
