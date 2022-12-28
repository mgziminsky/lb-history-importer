pub trait ListenData<'l> {
    type MetaType: serde::Serialize;

    fn listened_at(&'l self) -> i64;

    fn track_name(&'l self) -> &str;
    fn artist_name(&'l self) -> &str;

    fn release_name(&'l self) -> Option<&str> { None }

    fn track_metadata(&'l self) -> Option<Self::MetaType> { None }
}
pub trait IntoPayloadDerive: for<'a> ListenData<'a> {}
