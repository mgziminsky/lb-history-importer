use std::vec::IntoIter;

use ::listenbrainz::raw::request::Payload;
use serde::Deserialize;
use serde_json::Value;

pub mod listenbrainz;
pub mod spotify;


fn additional_info<T: serde::Serialize>(data: &T) -> Option<serde_json::Map<String, Value>> {
    serde_json::to_value(data).ok().and_then(|v| match v {
        Value::Object(mut m) => {
            m.insert("submission_client".to_owned(), "lb-history-importer".into());
            m.insert("submission_client_version".to_owned(), env!("CARGO_PKG_VERSION").into());
            Some(m)
        },
        _ => None,
    })
}


/// Deserialization wrapper for a `Vec<_>` that will skip errors
#[serde_with::serde_as]
#[derive(Deserialize)]
pub struct ListenVec<T: PayloadT>(#[serde_as(as = "serde_with::VecSkipError<_>")] Vec<T>);
impl<T: PayloadT> From<ListenVec<T>> for Vec<T> {
    #[inline]
    fn from(value: ListenVec<T>) -> Self { value.0 }
}
impl<T: PayloadT> IntoIterator for ListenVec<T> {
    type IntoIter = IntoIter<Self::Item>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

pub trait PayloadT: for<'d> serde::Deserialize<'d> + Into<Payload<String>> {}
impl<T: for<'d> serde::Deserialize<'d> + Into<Payload<String>>> PayloadT for T {}
