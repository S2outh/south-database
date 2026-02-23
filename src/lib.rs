use async_nats::Subscriber;
use futures::future::join_all;
use futures::StreamExt;

use questdb::ingress::{
    Protocol, Sender, SenderBuilder
};

mod telestion_parser;
pub mod config;

#[derive(Debug)]
pub enum DBError {
    ConnectNATS(async_nats::ConnectErrorKind),
    SubscribeNATS(async_nats::SubscribeError),
    ConnectDB(questdb::Error),
    SendDB(questdb::Error),
}

pub async fn run(config: config::DBConfig) -> Result<(), DBError> {
    let nats_client = async_nats::ConnectOptions::with_user_and_password(config.nats_user, config.nats_pwd)
        .connect(config.nats_address)
        .await.map_err(|e| DBError::ConnectNATS(e.kind()))?;

    let mut subscribers = Vec::new();
    for topic in config.topics {
        let subscription = nats_client.subscribe(topic).await.map_err(DBError::SubscribeNATS)?;
        let database_client = SenderBuilder::new(Protocol::Http, &config.db_address, 9000).build().map_err(DBError::ConnectDB)?;
        let subscriber = tokio::spawn(subscriber_loop(subscription, database_client));
        subscribers.push(subscriber);
    }
    for result in join_all(subscribers).await {
        result.unwrap().expect("client subscriber finished with non 0 exit code");
    }
    Ok(())
}

async fn subscriber_loop(mut subscription: Subscriber, mut database_client: Sender) -> Result<(), DBError> {
    while let Some(msg) = subscription.next().await {
        
        match telestion_parser::to_buffer(msg.subject.as_str(), msg.payload.as_ref()) {
            Ok(mut buffer) => database_client.flush(&mut buffer).map_err(DBError::SendDB)?,
            Err(e) => eprintln!("error parsing incoming message: {:?}", e),
        }
    }
    Ok(())
}

