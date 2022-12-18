use serde_json::Value;

mod spotify;
pub use spotify::*;


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
