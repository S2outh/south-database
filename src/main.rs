use nats_questdb_ingress::config::DBConfig;
use simple_config::Config;

#[tokio::main]
async fn main() {
    let mut config = DBConfig::new();
    config.parse_file("db.conf").expect("could not parse config file");
    config.parse_cli().expect("could not parse cli args");

    nats_questdb_ingress::run(config).await
        .expect("database service finished with non zero exit code");
}
