#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(feature = "debug") {
            println!("[DEBUG] {}", format!($($arg)*))
        }
    };
}

pub use debug;
