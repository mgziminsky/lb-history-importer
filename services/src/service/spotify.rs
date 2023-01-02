use std::borrow::Borrow;

use lb_importer_core::ListenData;
use lb_importer_derive::IntoPayload;
use serde::{
    ser::SerializeStruct,
    Deserialize,
};
use time::{
    format_description::{
        self,
        well_known::Rfc3339,
    },
    macros::format_description,
    OffsetDateTime,
    PrimitiveDateTime,
};

pub type SpotifyListenVec = super::ListenVec<SpotifyListen>;


/// Represents a single entry from a spotify history dump
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize, IntoPayload)]
pub struct SpotifyListen {
    #[serde(alias = "endTime", alias = "ts", deserialize_with = "parse_datetime")]
    time: OffsetDateTime,

    #[serde(alias = "offline_timestamp", default, deserialize_with = "parse_ms_to_sec")]
    offline_time: Option<i64>,

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
    fn listened_at(&self) -> i64 { self.offline_time.filter(|&ts| ts > 100).unwrap_or_else(|| self.time.unix_timestamp()) }

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


fn parse_datetime<'de, D>(de: D) -> Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    const SIMPLE_FMT: &[format_description::FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]");

    let val = String::deserialize(de)?;
    OffsetDateTime::parse(&val, &Rfc3339)
        .or_else(|_| PrimitiveDateTime::parse(&val, &SIMPLE_FMT).map(PrimitiveDateTime::assume_utc))
        .map_err(serde::de::Error::custom)
}

fn parse_ms_to_sec<'de, D>(de: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<i64>::deserialize(de).map(|o| o.map(|ts| ts / 1000))
}

#[cfg(test)]
mod tests;
