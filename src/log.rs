#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use std::fs::OpenOptions;
        use std::io::prelude::*;

        extern crate chrono;
        use chrono::Local;

        if cfg!(feature = "debug") {
            println!("[DEBUG] {}", format!($($arg)*))
        }

        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("repulse.log")
        .unwrap();

        writeln!(file,
            "[DEBUG] [{}] [{}] {}",
            if is_elevated::is_elevated() {"ADMIN"} else {"USER"},
            Local::now().format("%Y-%m-%d][%H:%M:%S"),
            format!($($arg)*)
        ).unwrap();
    }};
}

pub use debug;
