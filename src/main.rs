extern crate clokwerk;
extern crate tokio;
extern crate uuid;
extern crate warp;

use clokwerk::{Scheduler, TimeUnits};
use std::fs;
use std::io;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use warp::{http::Response, Filter};

const IMAGE_DIR: &str = "images/";

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

fn take_image() -> Vec<u8> {
    let mut command = base_camera_command();
    command.arg("-o");
    command.arg("-");
    command.stdout(Stdio::piped());

    let child = command.spawn().expect("Command failed to start");
    let output = child.wait_with_output().unwrap();
    return output.stdout;
}

fn gen_timelapse_filename() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

fn take_timelapse_photo() {
    let filename = gen_timelapse_filename();
    let filepath = format!("{}{}.jpg", IMAGE_DIR, filename);

    take_and_save_image(filepath);
}

fn timelapse_thread() {
    let mut scheduler = Scheduler::new();

    scheduler.every(30.minute()).run(|| take_timelapse_photo());

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(500));
    }
}

fn image_response(image: Vec<u8>) -> Result<Response<Vec<u8>>, warp::http::Error> {
    return Response::builder()
        .header("Content-Type", "image/jpeg")
        .body(image);
}

fn handle_latest() -> Result<Response<Vec<u8>>, warp::http::Error> {
    let mut entries = fs::read_dir(IMAGE_DIR)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.sort().reverse();

    let filepath = &entries[0];
    let image = fs::read(filepath).unwrap();

    return image_response(image);
}

fn handle_current() -> Result<Response<Vec<u8>>, warp::http::Error> {
    let image = take_image();
    return image_response(image);
}

#[tokio::main]
async fn main() {
    let timelapse_child = thread::spawn(timelapse_thread);

    let index = warp::get().and(warp::path::end()).map(|| handle_latest());
    let current = warp::path("current")
        .and(warp::path::end())
        .map(|| handle_current());
    let images = warp::path("images").and(warp::fs::dir(IMAGE_DIR));

    let routes = index.or(current).or(images);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    timelapse_child.join().unwrap();
}
