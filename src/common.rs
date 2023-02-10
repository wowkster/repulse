#[macro_export]
macro_rules! wcstr {
    ($string:literal) => {{
        use std::{ffi::OsStr, os::windows::prelude::OsStrExt};

        // Encode string as wide str
        let mut encoded: Vec<u16> = OsStr::new($string).encode_wide().collect();

        // encode_wide does NOT add a null terminator
        encoded.push(0);

        encoded
    }};
}

#[inline]
pub fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_be_bytes([0, b, g, r])
}

#[macro_export]
macro_rules! low_dword {
    ($name: ident) => {
        (($name & 0x0000FFFF) as u16)
    };
}

#[macro_export]
macro_rules! high_dword {
    ($name: ident) => {
        ((($name & 0xFFFF0000) >> 16) as u16)
    };
}
