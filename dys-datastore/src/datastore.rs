#![allow(async_fn_in_trait)]

pub trait Datastore {
    type DatastoreConfig;

    /// Connect to the datastore given a config
    async fn connect(config: Self::DatastoreConfig) -> Result<Box<Self>, ()>;

    /// Is this datastore object currently connected?
    async fn is_connected(&mut self) -> bool;
}