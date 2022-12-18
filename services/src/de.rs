use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use serde::{
    de::Error,
    Deserialize,
};


pub fn from_datetime_str<'de, D>(de: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = String::deserialize(de)?;
    val.parse()
        .or_else(|_| Utc.datetime_from_str(val.as_str(), "%Y-%m-%d %H:%M"))
        .map_err(Error::custom)
}
