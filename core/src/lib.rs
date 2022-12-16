pub trait ListenData {
    type MetaType: serde::Serialize;

    fn listened_at(&self) -> i64;

    fn track_name(&self) -> &str;
    fn artist_name(&self) -> &str;

    fn release_name(&self) -> Option<&str> { None }

    fn track_metadata(&self) -> Option<Self::MetaType> { None }
}
pub trait IntoPayloadDerive: ListenData {}
