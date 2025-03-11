use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NatsError {
    MalformedRequest,
    UnhandledRequest,
    InternalSerializationError,
}