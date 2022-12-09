use anyhow::anyhow;
use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use listenbrainz::raw::request::{
    Payload,
    StrType,
    TrackMetadata,
};
use serde::{
    ser::SerializeStruct,
    Deserialize,
};
use serde_json::Value;

const SIMPLE_DATE_FMT: &str = "%Y-%m-%d %H:%M";

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize)]
pub(crate) struct HistEntry {
    #[serde(deserialize_with = "from_datetime_str")]
    #[serde(alias = "endTime", alias = "ts")]
    time: DateTime<Utc>,

    #[serde(alias = "offline_timestamp", default, with = "chrono::serde::ts_milliseconds_option")]
    offline_time: Option<DateTime<Utc>>,

    #[serde(alias = "artistName", alias = "master_metadata_album_artist_name")]
    pub artist: Option<String>,

    #[serde(alias = "trackName", alias = "master_metadata_track_name")]
    pub track: Option<String>,

    #[serde(alias = "master_metadata_album_album_name")]
    pub album: Option<String>,

    pub spotify_track_uri: Option<String>,

    #[serde(alias = "msPlayed", alias = "ms_played")]
    pub ms_played: u32,
}

impl HistEntry {
    pub fn time(&self) -> DateTime<Utc> {
        self.offline_time.filter(|dt| dt.timestamp() > 0).unwrap_or(self.time)
    }
}

impl TryFrom<HistEntry> for Payload<String, String, String> {
    type Error = anyhow::Error;

    fn try_from(val: HistEntry) -> Result<Self, Self::Error> {
        let spotify_id = val
            .spotify_track_uri
            .as_ref()
            .and_then(|uri| uri.rsplit_once(':'))
            .map(|(_, id)| format!("https://open.spotify.com/tracks/{id}"));

        Ok(Payload {
            listened_at: Some(val.time().timestamp()),
            track_metadata: TrackMetadata {
                track_name: val.track.ok_or(anyhow!("Missing track name"))?,
                artist_name: val.artist.ok_or(anyhow!("Missing artist name"))?,
                release_name: val.album,
                additional_info: serde_json::to_value(Info { spotify_id })
                    .and_then(|v| match v {
                        Value::Object(m) => Ok(m),
                        _ => unreachable!(),
                    })
                    .ok(),
            },
        })
    }
}

#[derive(Debug)]
struct Info<T: StrType> {
    spotify_id: Option<T>,
}

impl<T: StrType> serde::Serialize for Info<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("additional_info", 5)?;
        state.serialize_field("music_service", "spotify.com")?;
        state.serialize_field("submission_client", env!("CARGO_PKG_NAME"))?;
        state.serialize_field("submission_client_version", env!("CARGO_PKG_VERSION"))?;
        if let Some(id) = self.spotify_id.as_ref() {
            state.serialize_field("spotify_id", id)?;
            state.serialize_field("origin_url", id)?;
        } else {
            state.skip_field("spotify_id")?;
            state.skip_field("origin_url")?;
        }
        state.end()
    }
}

fn from_datetime_str<'de, D>(de: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = String::deserialize(de)?;
    val.parse()
        .or_else(|_| Utc.datetime_from_str(val.as_str(), SIMPLE_DATE_FMT))
        .map_err(serde::de::Error::custom)
}


#[cfg(test)]
mod test {
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

    const EXPECTED_TRACK: Option<&str> = Some("Burn Brighter");
    const EXPECTED_ARTIST: Option<&str> = Some("Lansdowne");
    const EXPECTED_ALBUM: Option<&str> = Some("No Home but the Road");
    const EXPECTED_MS_PLAYED: u32 = 60265;

    #[test]
    fn test_de_simple() {
        let expected = HistEntry {
            time: Utc.with_ymd_and_hms(2018, 7, 10, 6, 58, 0).unwrap(),
            offline_time: None,
            track: EXPECTED_TRACK.map(str::to_owned),
            artist: EXPECTED_ARTIST.map(str::to_owned),
            album: None,
            spotify_track_uri: None,
            ms_played: EXPECTED_MS_PLAYED,
        };

        let simple: HistEntry = serde_json::from_str(SIMPLE_SAMPLE).expect("Failed to parse simple entry");
        assert_eq!(simple, expected);
    }

    #[test]
    fn test_de_full() {
        let expected = HistEntry {
            time: Utc.with_ymd_and_hms(2018, 7, 10, 6, 58, 55).unwrap(),
            offline_time: Utc.timestamp_millis_opt(1531090963961).single(),
            track: EXPECTED_TRACK.map(str::to_owned),
            artist: EXPECTED_ARTIST.map(str::to_owned),
            album: EXPECTED_ALBUM.map(str::to_owned),
            spotify_track_uri: Some("spotify:track:6BUMVGOnIeOIE6YetJGGDT".to_owned()),
            ms_played: EXPECTED_MS_PLAYED,
        };

        let full: HistEntry = serde_json::from_str(FULL_SAMPLE).expect("Failed to parse full entry");
        assert_eq!(full, expected);
    }
}
