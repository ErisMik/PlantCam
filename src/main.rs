extern crate clokwerk;

use clokwerk::{Scheduler, TimeUnits};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn gen_filename() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

fn take_picture() {
    let filename = gen_filename();

    let mut command = Command::new("raspistill");
    command.arg("-o");
    command.arg(format!("images/{}.jpg", filename));

    let mut child = command.spawn().expect("Command failed to start");
    let _ = child.wait().unwrap();
}

fn main() {
    let mut scheduler = Scheduler::new();

    scheduler.every(1.hour()).run(|| take_picture());

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(500));
    }
}
