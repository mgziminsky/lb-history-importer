#![feature(once_cell)]

use std::{
    ffi::OsStr,
    fs::File,
    io::BufReader,
    path::Path,
    sync::LazyLock,
};

use anyhow::{
    anyhow,
    Result,
};
pub use lb_importer_core::*;
use regex::Regex;

use crate::service::{
    LBListenVec,
    SpotifyListenVec,
};

pub mod service;


pub enum ImportData {
    Spotify(SpotifyListenVec),
    ListenBrainz(LBListenVec),
}

pub fn load_listens(path: &Path) -> Result<ImportData> {
    const ERR_MSG: &str = "Unrecognized file name";
    static SPOTIFY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^(endsong_|StreamingHistory)\d+\b"#).unwrap());
    static LB_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\w+_lb-\d{4}-\d{2}-\d{2}\b"#).unwrap());

    #[rustfmt::skip]
    macro_rules! parse_listens {
        ($rdr:expr, $ty:path) => { serde_json::from_reader($rdr).map($ty).map_err(Into::into) }
    }

    let name = path.file_stem().and_then(OsStr::to_str).ok_or(anyhow!(ERR_MSG))?;
    let rdr = File::open(path).map(BufReader::new)?;
    if SPOTIFY_REGEX.is_match(name) {
        parse_listens!(rdr, ImportData::Spotify)
    } else if LB_REGEX.is_match(name) {
        parse_listens!(rdr, ImportData::ListenBrainz)
    } else {
        Err(anyhow!(ERR_MSG))
    }
}
