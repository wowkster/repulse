use std::{fs, io, path::Path};

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

pub fn ensure_path_and_write<P, C>(path: P, contents: C) -> io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = path.as_ref();
    let contents = contents.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;

    Ok(())
}
