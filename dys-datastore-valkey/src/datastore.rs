pub use redis::aio::MultiplexedConnection;
pub use redis::AsyncCommands;
pub use redis::AsyncIter;
pub use redis::ExpireOption;

use redis::{Client, ConnectionAddr, ConnectionInfo, IntoConnectionInfo, ProtocolVersion, RedisConnectionInfo, RedisResult};
use dys_datastore::datastore::Datastore;

#[derive(Debug)]
pub struct ValkeyDatastore {
    connection: MultiplexedConnection
}

#[derive(Debug)]
pub struct ValkeyConfig {
    user: String,
    pass: String,
    host: String,
    port: u16,
}

impl ValkeyConfig {
    pub fn new(
        user: impl Into<String>,
        pass: impl Into<String>,
        host: impl Into<String>,
        port: u16
    ) -> ValkeyConfig {
        ValkeyConfig {
            user: user.into(),
            pass: pass.into(),
            host: host.into(),
            port,
        }
    }
}

impl IntoConnectionInfo for ValkeyConfig {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        Ok(ConnectionInfo {
            addr: ConnectionAddr::Tcp(self.host, self.port),
            redis: RedisConnectionInfo {
                db: 0,
                username: Some(self.user),
                password: Some(self.pass),
                protocol: ProtocolVersion::default(),
            }
        })
    }
}

impl ValkeyDatastore {
    pub fn connection(&mut self) -> MultiplexedConnection {
        self.connection.clone()
    }
}

impl Datastore for ValkeyDatastore {
    type DatastoreConfig = ValkeyConfig;

    async fn connect(config: Self::DatastoreConfig) -> Result<Self, ()> {
        let client = Client::open(config).unwrap(); // ZJ-TODO: handle this
        let config = redis::AsyncConnectionConfig::new();
        let connection = client
            .get_multiplexed_async_connection_with_config(&config)
            .await
            .unwrap();

        Ok(ValkeyDatastore {
            connection
        })
    }

    async fn check_connection(&mut self) -> bool {
        self.connection.ping::<String>().await.is_ok()
    }
}