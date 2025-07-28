use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use serde::ser::SerializeSeq;
use serde::Serializer;
use crate::games::instance::GameInstance;
use crate::season::series::SeriesGameIndex;

pub(crate) fn serialize_game_instances<S>(
    games: &BTreeMap<SeriesGameIndex, Arc<Mutex<GameInstance>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(games.len()))?;
    for (_, game_instance) in games {
        seq.serialize_element(&*game_instance.lock().unwrap())?;
    }
    seq.end()
}
