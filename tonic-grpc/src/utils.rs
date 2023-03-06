use std::env;
use std::ffi::OsStr;
use std::path::Path;

pub fn bin_name() -> Option<String> {
    env::args()
        .next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}
