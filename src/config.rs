use simple_config::Config;

#[derive(Config)]
pub struct DBConfig {
    // -- Nats
    pub nats_address: String,
    pub nats_user: String,
    pub nats_pwd: String,
    pub topics: Vec<String>,
    // -- DB
    pub db_address: String,
}
impl DBConfig {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        Self {
            nats_address: String::from("127.0.0.1"),
            nats_user: String::from("nats"),
            nats_pwd: String::from("nats"),
            topics: vec![">".into()],
            db_address: String::from("127.0.0.1"),
        }
    }
}
