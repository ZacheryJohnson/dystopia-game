use std::fmt::{Debug, Display};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum NatsError {
    MalformedRequest,
    InternalSerializationError,
    ReplySubjectSubscribeError,
    PublishError,
    PublishTimeout,
}

impl Display for NatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}