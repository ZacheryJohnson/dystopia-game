use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NatsError {
    MalformedRequest,
    UnhandledRequest,
    InternalSerializationError,
    ReplySubjectSubscribeError,
    PublishError,
    PublishTimeout,
}

impl std::fmt::Display for NatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}