use async_nats::Subscriber;
use futures::future::join_all;
use futures::StreamExt;

use anyhow::{Context, Result};

use env_logger;

use questdb::ingress::{
    Protocol, Sender, SenderBuilder
};

mod telestion_parser;
pub mod config;

const QUESTDB_PORT: u16 = 9000;

pub async fn run(config: config::DBConfig) -> Result<()> {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .format_timestamp_micros()
        .init();

    log::info!("Connecting to Nats...");

    let nats_client = async_nats::ConnectOptions::with_user_and_password(config.nats_user, config.nats_pwd)
        .connect(config.nats_address)
        .await?;

    log::info!("Nats connection successfull!");

    let mut subscribers = Vec::new();
    for topic in config.topics {
        log::info!("Subscribing to: {}", topic);

        let subscription = nats_client.subscribe(topic).await?;
        let database_client = SenderBuilder::new(Protocol::Http, &config.db_address, QUESTDB_PORT).build()?;
        let subscriber = tokio::spawn(subscriber_loop(subscription, database_client));
        subscribers.push(subscriber);
    }
    for result in join_all(subscribers).await {
        result?.context("client subscriber finished with non 0 exit code")?;
    }
    Ok(())
}

async fn subscriber_loop(mut subscription: Subscriber, mut database_client: Sender) -> Result<()> {
    while let Some(msg) = subscription.next().await {
        
        log::debug!("received msg with subject: {}", msg.subject);
        match telestion_parser::to_buffer(msg.subject.as_str(), msg.payload.as_ref()) {
            Ok(mut buffer) => database_client.flush(&mut buffer)?,
            Err(e) => log::error!("parsing incoming message: {:?}", e),
        }
    }
    Ok(())
}

