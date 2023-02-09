use std::{ffi::OsStr, os::windows::prelude::OsStrExt};

pub fn encode_wide_string(string: &str) -> Vec<u16> {
    // Encode string as wide str
    let mut encoded: Vec<u16> = OsStr::new(string).encode_wide().collect();

    // encode_wide does NOT add a null terminator
    encoded.push(0);

    encoded
}

pub fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    let mut res = 0u32;
    res |= b as u32;
    res <<= 8;
    res |= g as u32;
    res <<= 8;
    res |= r as u32;

    res
}

#[macro_export]
macro_rules! low_word {
    ($name: ident) => {
        (($name & 0x0000FFFF) as u16)
    };
}

pub use low_word;

#[macro_export]
macro_rules! high_word {
    ($name: ident) => {
        (($name & 0xFFFF0000) as u16)
    };
}

pub use high_word;
