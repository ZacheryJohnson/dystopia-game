#![allow(async_fn_in_trait)]

pub trait Datastore: Sized {
    type DatastoreConfig;

    /// Connect to the datastore given a config
    async fn connect(config: Self::DatastoreConfig) -> Result<Self, ()>;

    /// Is this datastore object currently connected?
    async fn check_connection(&mut self) -> bool;
}