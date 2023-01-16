pub use lb_importer_core::*;

use crate::service::{
    listenbrainz::ListenVec as LBListenVec,
    spotify::ListenVec as SpotifyListenVec,
};

pub mod service;

macro_rules! load_fn {
    ($name:ident, $ty:path) => {
        pub fn $name(source: impl std::io::Read) -> anyhow::Result<$ty> { serde_json::from_reader(source).map_err(Into::into) }
    };
}

load_fn!(load_spotify, SpotifyListenVec);
load_fn!(load_listenbrainz, LBListenVec);
