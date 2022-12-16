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
#[derive(Deserialize)]
pub struct SpotifyListenVec(#[serde(deserialize_with = "vec_skip_errors")] Vec<SpotifyListen>);

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

impl ListenData for SpotifyListen {
    type MetaType = Info;

    fn listened_at(&self) -> i64 {
        self.offline_time
            .filter(|dt| dt.timestamp() > 0)
            .unwrap_or(self.time)
            .timestamp()
    }

    fn track_name(&self) -> &str { self.track.borrow() }

    fn artist_name(&self) -> &str { self.artist.borrow() }

    fn release_name(&self) -> Option<&str> { self.album.as_deref() }

    fn track_metadata(&self) -> Option<Info> {
        self.spotify_track_uri
            .as_ref()
            .and_then(|uri| uri.rsplit_once(':'))
            .map(|(_, id)| format!("https://open.spotify.com/tracks/{id}"))
            .map(|s| Info { spotify_id: s })
    }
}

pub struct Info {
    spotify_id: String,
}

impl serde::Serialize for Info {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("additional_info", 3)?;
        state.serialize_field("music_service", "spotify.com")?;
        state.serialize_field("spotify_id", self.spotify_id.as_str())?;
        state.serialize_field("origin_url", self.spotify_id.as_str())?;
        state.end()
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::*;

    const FULL_SAMPLE: &str = r#"{
        "ts": "2018-07-10T06:58:55Z",
        "username": "0000000000",
        "platform": "iOS 11.4 (iPhone6,1)",
        "ms_played": 60265,
        "conn_country": "US",
        "ip_addr_decrypted": "0.0.0.0",
        "user_agent_decrypted": "unknown",
        "master_metadata_track_name": "Burn Brighter",
        "master_metadata_album_artist_name": "Lansdowne",
        "master_metadata_album_album_name": "No Home but the Road",
        "spotify_track_uri": "spotify:track:6BUMVGOnIeOIE6YetJGGDT",
        "episode_name": null,
        "episode_show_name": null,
        "spotify_episode_uri": null,
        "reason_start": "trackdone",
        "reason_end": "fwdbtn",
        "shuffle": true,
        "skipped": null,
        "offline": true,
        "offline_timestamp": 1531090963961,
        "incognito_mode": false
    }"#;

    const SIMPLE_SAMPLE: &str = r#"{
        "endTime" : "2018-07-10 06:58",
        "trackName" : "Burn Brighter",
        "artistName" : "Lansdowne",
        "msPlayed" : 60265
    }"#;

    const EXPECTED_TRACK: &str = "Burn Brighter";
    const EXPECTED_ARTIST: &str = "Lansdowne";
    const EXPECTED_ALBUM: Option<&str> = Some("No Home but the Road");
    const EXPECTED_MS_PLAYED: u32 = 60265;

    #[test]
    fn test_de_simple() {
        let expected = SpotifyListen {
            time: Utc.with_ymd_and_hms(2018, 7, 10, 6, 58, 0).unwrap(),
            offline_time: None,
            track: EXPECTED_TRACK.to_owned(),
            artist: EXPECTED_ARTIST.to_owned(),
            album: None,
            spotify_track_uri: None,
            ms_played: EXPECTED_MS_PLAYED,
        };

        let simple: SpotifyListen = serde_json::from_str(SIMPLE_SAMPLE).expect("Failed to parse simple entry");
        assert_eq!(simple, expected);
    }

    #[test]
    fn test_de_full() {
        let expected = SpotifyListen {
            time: Utc.with_ymd_and_hms(2018, 7, 10, 6, 58, 55).unwrap(),
            offline_time: Utc.timestamp_millis_opt(1531090963961).single(),
            track: EXPECTED_TRACK.to_owned(),
            artist: EXPECTED_ARTIST.to_owned(),
            album: EXPECTED_ALBUM.map(str::to_owned),
            spotify_track_uri: Some("spotify:track:6BUMVGOnIeOIE6YetJGGDT".to_owned()),
            ms_played: EXPECTED_MS_PLAYED,
        };

        let full: SpotifyListen = serde_json::from_str(FULL_SAMPLE).expect("Failed to parse full entry");
        assert_eq!(full, expected);
    }
}
