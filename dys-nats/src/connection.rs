pub struct ConnectionConfig {
    host: Option<String>,
    port: Option<u16>,
    token: Option<String>,
}

impl ConnectionConfig {
    #[must_use]
    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    #[must_use]
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn into_connection_string(self) -> String {
        format!("{}:{}", self.host.unwrap(), self.port.unwrap())
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        ConnectionConfig {
            host: Some(std::env::var("NATS_HOST").unwrap_or(String::from("172.18.0.1"))),
            port: Some(std::env::var("NATS_PORT").unwrap_or(String::from("4222")).parse::<u16>().unwrap()),
            token: Some(std::env::var("NATS_TOKEN").unwrap_or(String::from("replaceme"))),
        }
    }
}

pub async fn make_client(config: ConnectionConfig) -> async_nats::Client {
    async_nats::ConnectOptions::new()
        .token(config.get_token().unwrap())
        .connect(config.into_connection_string())
        .await
        .unwrap()
}