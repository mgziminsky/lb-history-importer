use std::marker::PhantomData;

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use serde::{
    de::{
        Error,
        SeqAccess,
        Visitor,
    },
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

struct GoodVisitor<'de, T: Deserialize<'de>>(PhantomData<T>, PhantomData<&'de T>);
impl<'de, T: Deserialize<'de>> Visitor<'de> for GoodVisitor<'de, T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result { formatter.write_str("a sequence") }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        loop {
            match seq.next_element() {
                Ok(Some(listen)) => values.push(listen),
                Ok(None) => break,
                Err(e) => eprintln!("{e:#}"),
            }
        }
        Ok(values)
    }
}

pub fn vec_skip_errors<'de, D, T>(de: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + 'de,
{
    de.deserialize_seq(GoodVisitor(PhantomData, PhantomData))
}
