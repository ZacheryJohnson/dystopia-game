use std::sync::{Arc, Mutex};
use serde::ser::SerializeSeq;
use serde::Serializer;
use crate::matches::instance::MatchInstance;

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
