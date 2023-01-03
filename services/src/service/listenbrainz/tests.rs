use std::collections::HashMap;

use serde_json::Value::String;

use super::*;

const SAMPLE: &str = r#"{
    "track_metadata": {
        "artist_name": "The Cab",
        "release_name": "Symphony Soldier",
        "additional_info": {
            "origin_url": "https://open.spotify.com/tracks/49rpdsNYJirTTf6p6mMvag",
            "spotify_id": "https://open.spotify.com/tracks/49rpdsNYJirTTf6p6mMvag",
            "music_service": "spotify.com",
            "recording_msid": "7407a60c-ba0e-4fcd-ba47-80194f002b20",
            "submission_client": "spotify-importer",
            "submission_client_version": "0.1.0"
        },
        "track_name": "Angel With A Shotgun",
        "mbid_mapping": {
            "recording_mbid": "b92334c4-574a-46f5-89d8-417fcd1e873f",
            "release_mbid": "c40291c6-4a66-4d96-a8a2-d144205c61b0",
            "artist_mbids": [
                "91f7a868-d82e-4cfb-9cd9-a2ffd7faac25"
            ],
            "artists": [
                {
                    "artist_mbid": "91f7a868-d82e-4cfb-9cd9-a2ffd7faac25",
                    "artist_credit_name": "The Cab",
                    "join_phrase": ""
                }
            ],
            "caa_id": 1487209368,
            "caa_release_mbid": "f5663634-2bd6-4b01-beab-1d979a599da0"
        }
    },
    "listened_at": 1669318360,
    "recording_msid": "7407a60c-ba0e-4fcd-ba47-80194f002b20",
    "user_name": "zozCXAEwpVLa",
    "inserted_at": 1670569910
}"#;

#[test]
fn test_de() {
    let expected = {
        #[rustfmt::skip]
        let additional_info = {
            let mut data = HashMap::new();
            data.insert("submission_client".to_owned(), String("spotify-importer".to_owned()));
            data.insert("submission_client_version".to_owned(), String("0.1.0".to_owned()));
            data.insert("music_service".to_owned(), String("spotify.com".to_owned()));
            data.insert("spotify_id".to_owned(), String("https://open.spotify.com/tracks/49rpdsNYJirTTf6p6mMvag".to_owned()));
            data.insert("origin_url".to_owned(), String("https://open.spotify.com/tracks/49rpdsNYJirTTf6p6mMvag".to_owned()));
            data.insert("recording_msid".to_owned(), String("7407a60c-ba0e-4fcd-ba47-80194f002b20".to_owned()));
            data
        };
        Listen {
            track_metadata: AdditionalInfo {
                data: UserListensTrackMetadata {
                    artist_name: "The Cab".to_owned(),
                    track_name: "Angel With A Shotgun".to_owned(),
                    release_name: Some("Symphony Soldier".to_owned()),
                    additional_info,
                },
                mbid_mapping: MbidMapping {
                    recording_mbid: "b92334c4-574a-46f5-89d8-417fcd1e873f".to_owned(),
                    release_mbid: "c40291c6-4a66-4d96-a8a2-d144205c61b0".to_owned(),
                    artist_mbids: vec!["91f7a868-d82e-4cfb-9cd9-a2ffd7faac25".to_owned()],
                },
            },
            listened_at: 1_669_318_360,
        }
    };

    let listen: Listen = serde_json::from_str(SAMPLE).expect("Failed to parse listen");
    assert_eq!(listen, expected);
}


impl PartialEq for AdditionalInfo {
    fn eq(&self, other: &Self) -> bool {
        self.mbid_mapping == other.mbid_mapping
            && self.data.track_name == other.data.track_name
            && self.data.artist_name == other.data.artist_name
            && self.data.release_name == other.data.release_name
            && self.data.additional_info == other.data.additional_info
    }
}
impl Eq for AdditionalInfo {}
