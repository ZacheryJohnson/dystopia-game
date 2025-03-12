use std::fmt::Debug;
use std::pin::Pin;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::NatsError;

pub trait NatsRpcServer: tower::Service<
    async_nats::Message,
    Error=NatsError,
    Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
    Response=Bytes,
> {
    type Request: Serialize + DeserializeOwned + Debug;
    type Response: Serialize + DeserializeOwned + Debug;

    const RPC_SUBJECT: &'static str;
    fn rpc_subject(&self) -> &'static str { Self::RPC_SUBJECT }
}