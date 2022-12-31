use lb_importer_core::ListenData;
use lb_importer_derive::IntoPayload;
use listenbrainz::raw::response::UserListensTrackMetadata;
use serde::{
    ser::SerializeMap,
    Deserialize,
    Serialize,
};

use super::ListenVec;

pub type LBListenVec = ListenVec<LBListen>;

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize, IntoPayload)]
#[payload(track = track_metadata.data.track_name: String)]
#[payload(artist = track_metadata.data.artist_name: String)]
#[payload(release = track_metadata.data.release_name: String)]
pub struct LBListen {
    track_metadata: AdditionalInfo,
    listened_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdditionalInfo {
    #[serde(flatten)]
    data: UserListensTrackMetadata,
    mbid_mapping: MbidMapping,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize)]
struct MbidMapping {
    recording_mbid: String,
    release_mbid: String,
    artist_mbids: Vec<String>,
}

impl Serialize for AdditionalInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut info = serializer.serialize_map(Some(self.data.additional_info.len() + 3))?;
        self.data.additional_info.iter().try_for_each(|(k, v)| info.serialize_entry(k, v))?;
        info.serialize_entry("recording_mbid", self.mbid_mapping.recording_mbid.as_str())?;
        info.serialize_entry("release_mbid", self.mbid_mapping.release_mbid.as_str())?;
        info.serialize_entry("artist_mbids", self.mbid_mapping.artist_mbids.as_slice())?;
        info.end()
    }
}

impl<'l> ListenData<'l> for LBListen {
    type MetaType = &'l AdditionalInfo;

    fn listened_at(&self) -> i64 { self.listened_at }

    fn track_name(&'l self) -> &str { self.track_metadata.data.track_name.as_str() }

    fn artist_name(&'l self) -> &str { self.track_metadata.data.artist_name.as_str() }

    fn release_name(&'l self) -> Option<&str> { self.track_metadata.data.release_name.as_deref() }

    fn track_metadata(&'l self) -> Option<Self::MetaType> { Some(&self.track_metadata) }
}


#[cfg(test)]
mod tests;
