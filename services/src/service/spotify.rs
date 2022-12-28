use std::borrow::Borrow;

use chrono::{
    DateTime,
    Utc,
};
use lb_importer_core::ListenData;
use lb_importer_derive::IntoPayload;
use serde::{
    ser::SerializeStruct,
    Deserialize,
};

use crate::de::*;


/// Deserialization wrapper for a `Vec<SpotifyListen>` that will skip errors
#[serde_with::serde_as]
#[derive(Deserialize)]
pub struct SpotifyListenVec(#[serde_as(as = "serde_with::VecSkipError<_>")] Vec<SpotifyListen>);

impl From<SpotifyListenVec> for Vec<SpotifyListen> {
    #[inline]
    fn from(value: SpotifyListenVec) -> Self { value.0 }
}

/// Represents a single entry from a spotify history dump
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize, IntoPayload)]
pub struct SpotifyListen {
    #[serde(deserialize_with = "from_datetime_str")]
    #[serde(alias = "endTime", alias = "ts")]
    time: DateTime<Utc>,

    #[serde(alias = "offline_timestamp", default, with = "chrono::serde::ts_milliseconds_option")]
    offline_time: Option<DateTime<Utc>>,

    #[serde(alias = "trackName", alias = "master_metadata_track_name")]
    pub track: String,

    #[serde(alias = "artistName", alias = "master_metadata_album_artist_name")]
    pub artist: String,

    #[release]
    #[serde(alias = "master_metadata_album_album_name")]
    pub album: Option<String>,

    pub spotify_track_uri: Option<String>,

    #[serde(alias = "msPlayed", alias = "ms_played")]
    pub ms_played: u32,
}

impl ListenData<'_> for SpotifyListen {
    type MetaType = Info;

    #[inline]
    fn listened_at(&self) -> i64 {
        self.offline_time
            .filter(|dt| dt.timestamp() > 0)
            .unwrap_or(self.time)
            .timestamp()
    }

    #[inline]
    fn track_name(&self) -> &str { self.track.borrow() }

    #[inline]
    fn artist_name(&self) -> &str { self.artist.borrow() }

    #[inline]
    fn release_name(&self) -> Option<&str> { self.album.as_deref() }

    #[inline]
    fn track_metadata(&self) -> Option<Info> {
        Some(Info {
            spotify_id: self
                .spotify_track_uri
                .as_ref()
                .and_then(|uri| uri.rsplit_once(':'))
                .map(|(_, id)| format!("https://open.spotify.com/tracks/{id}")),
        })
    }
}

pub struct Info {
    spotify_id: Option<String>,
}

impl serde::Serialize for Info {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("additional_info", 3)?;
        state.serialize_field("music_service", "spotify.com")?;
        if let Some(ref id) = self.spotify_id {
            state.serialize_field("spotify_id", id.as_str())?;
            state.serialize_field("origin_url", id.as_str())?;
        } else {
            state.skip_field("spotify_id")?;
            state.skip_field("origin_url")?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests;
