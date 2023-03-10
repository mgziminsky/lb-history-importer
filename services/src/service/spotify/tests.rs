use time::macros::datetime;

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

macro_rules! LIST_SAMPLE {
    () => {
        format!("[{0},{{}},{0},{1},{{}},{1}]", SIMPLE_SAMPLE, FULL_SAMPLE)
    };
}

const EXPECTED_TRACK: &str = "Burn Brighter";
const EXPECTED_ARTIST: &str = "Lansdowne";
const EXPECTED_ALBUM: Option<&str> = Some("No Home but the Road");
const EXPECTED_MS_PLAYED: u32 = 60265;

#[test]
fn test_de_simple() {
    let expected = Listen {
        time: datetime!(2018-07-10 06:58 UTC),
        offline_time: None,
        track: EXPECTED_TRACK.to_owned(),
        artist: EXPECTED_ARTIST.to_owned(),
        album: None,
        spotify_track_uri: None,
        ms_played: EXPECTED_MS_PLAYED,
        reason_start: None,
        reason_end: None,
    };

    let simple: Listen = serde_json::from_str(SIMPLE_SAMPLE).expect("Failed to parse simple entry");
    assert_eq!(simple, expected);
}

#[test]
fn test_de_full() {
    let expected = Listen {
        time: datetime!(2018-07-10 06:58:55 UTC),
        offline_time: Some(1_531_090_963),
        track: EXPECTED_TRACK.to_owned(),
        artist: EXPECTED_ARTIST.to_owned(),
        album: EXPECTED_ALBUM.map(str::to_owned),
        spotify_track_uri: Some("spotify:track:6BUMVGOnIeOIE6YetJGGDT".to_owned()),
        ms_played: EXPECTED_MS_PLAYED,
        reason_start: Some("trackdone".to_owned()),
        reason_end: Some("fwdbtn".to_owned()),
    };

    let full: Listen = serde_json::from_str(FULL_SAMPLE).expect("Failed to parse full entry");
    assert_eq!(full, expected);
}

#[test]
fn test_de_list_safe() {
    let list: ListenVec = serde_json::from_str(LIST_SAMPLE!().as_str()).expect("Failed to parse list");
    assert_eq!(list.0.len(), 4);
}

#[test]
#[should_panic(expected = "missing field")]
fn test_de_list_fail() { serde_json::from_str::<Vec<Listen>>(LIST_SAMPLE!().as_str()).expect(""); }
