use std::sync::{Arc, Mutex};
use serde::de::DeserializeSeed;
use serde::ser::SerializeSeq;
use serde::{Deserializer, Serializer};
use crate::matches::instance::MatchInstance;
use crate::schedule::series::Series;

pub(crate) fn serialize_match_instances<S>(
    matches: &Vec<Arc<Mutex<MatchInstance>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(matches.len()))?;
    for match_instance in matches {
        seq.serialize_element(&*match_instance.lock().unwrap())?;
    }
    seq.end()
}
