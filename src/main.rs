use anyhow::Context;
use nats_questdb_ingress::config::DBConfig;
use simple_config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = DBConfig::new();
    config.parse_file("db.conf").context("could not parse config file")?;
    config.parse_cli().context("could not parse cli args")?;

    nats_questdb_ingress::run(config).await
        .context("database service finished with non zero exit code")?;

    Ok(())
}
