extern crate clokwerk;
extern crate tokio;
extern crate warp;

use clokwerk::{Scheduler, TimeUnits};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use warp::Filter;

fn base_camera_command() -> std::process::Command {
    let mut command = Command::new("raspistill");
    command.arg("-vf");
    command.arg("-hf");
    command.arg("-n");
    return command;
}

fn gen_timelapse_filename() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

fn take_timelapse_photo() {
    let filename = gen_timelapse_filename();

    let mut command = base_camera_command();
    command.arg("-o");
    command.arg(format!("images/{}.jpg", filename));

    let mut child = command.spawn().expect("Command failed to start");
    let _ = child.wait().unwrap();
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
    let mut command = base_camera_command();
    command.arg("-o");
    command.arg("-");

    let child = command.spawn().expect("Command failed to start");
    let result = child.wait_with_output().unwrap();
    let raw_image = result.stdout;

    return raw_image;
}

#[tokio::main]
async fn main() {
    let timelapse_child = thread::spawn(timelapse_thread);

    let index = warp::get()
        .and(warp::path::end())
        .map(|| take_instant_photo());
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let routes = index.or(hello);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    timelapse_child.join().unwrap();
}
