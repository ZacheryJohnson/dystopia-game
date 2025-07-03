use std::sync::{Arc, Mutex};
use serde::Serializer;
use crate::arena::Arena;

pub(crate) fn _serialize_arena_to_id<S>(
    _: &Arc<Mutex<Arena>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(0)
    // ZJ-TODO
    // s.serialize_u64(arena.lock().unwrap().id)
}