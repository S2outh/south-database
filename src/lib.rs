use async_nats::Subscriber;
use futures::future::join_all;
use futures::StreamExt;

use std::{fmt, error};

use questdb::ingress::{
    Protocol, Sender, SenderBuilder
};

#[macro_use]
mod macros {
    macro_rules! from_inner_err {
        ($outer: ty, $($variant: ident, $inner: ty,)*) => {
            $(impl From<$inner> for $outer {
                fn from(value: $inner) -> Self {
                    Self::$variant(value)
                }
            })*
        };
    }
}

mod telestion_parser;
pub mod config;

#[derive(Debug)]
pub enum DBError {
    ConnectNATS(async_nats::ConnectError),
    SubscribeNATS(async_nats::SubscribeError),
    QuestDBErr(questdb::Error),
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectNATS(e) => write!(f, "Nats connection error: {}", e),
            Self::SubscribeNATS(e) => write!(f, "Nats subscription error: {}", e),
            Self::QuestDBErr(e) => write!(f, "DB error: {}", e),
        }
    }
}

from_inner_err!(DBError,
    ConnectNATS, async_nats::ConnectError,
    SubscribeNATS, async_nats::SubscribeError,
    QuestDBErr, questdb::Error,
);

impl error::Error for DBError {}

type Result<T> = std::result::Result<T, DBError>;

pub async fn run(config: config::DBConfig) -> Result<()> {
    let nats_client = async_nats::ConnectOptions::with_user_and_password(config.nats_user, config.nats_pwd)
        .connect(config.nats_address)
        .await?;

    let mut subscribers = Vec::new();
    for topic in config.topics {
        let subscription = nats_client.subscribe(topic).await?;
        let database_client = SenderBuilder::new(Protocol::Http, &config.db_address, 9000).build()?;
        let subscriber = tokio::spawn(subscriber_loop(subscription, database_client));
        subscribers.push(subscriber);
    }
    for result in join_all(subscribers).await {
        result.unwrap().expect("client subscriber finished with non 0 exit code");
    }
    Ok(())
}

async fn subscriber_loop(mut subscription: Subscriber, mut database_client: Sender) -> Result<()> {
    while let Some(msg) = subscription.next().await {
        
        match telestion_parser::to_buffer(msg.subject.as_str(), msg.payload.as_ref()) {
            Ok(mut buffer) => database_client.flush(&mut buffer)?,
            Err(e) => eprintln!("error parsing incoming message: {:?}", e),
        }
    }
    Ok(())
}

