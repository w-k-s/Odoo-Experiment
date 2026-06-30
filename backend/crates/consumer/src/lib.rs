use loyalty::LoyaltyEngine;
use rdkafka::Message;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, DefaultConsumerContext};
use rdkafka::error::KafkaError;
use rdkafka::message::Headers;
use std::sync::Arc;
use tracing::{info, warn};
use utils::config::ConsumerConfig;

pub async fn run(
    _loyalty_engine: Arc<dyn LoyaltyEngine>,
    consumer_config: ConsumerConfig,
) -> Result<(), KafkaError> {
    info!("Running consumer");
    let mut config = ClientConfig::new();

    config
        .set("group.id", &consumer_config.group_id)
        .set("bootstrap.servers", &consumer_config.bootstrap_servers)
        .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Debug);

    let consumer: StreamConsumer<DefaultConsumerContext> = config.create()?;
    let topics: Vec<&str> = consumer_config.topics.iter().map(String::as_str).collect();
    consumer.subscribe(&topics)?;

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Kafka Error: {e}"),
            Ok(m) => {
                info!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );
                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let h = headers.get(i);
                        info!("  Header {:#?}: {:?}", h.key, h.value);
                    }
                }

                // TODO:
                // 1. Match topic
                // 2. Depending on topic, parse payload to approp type
                // 3. Process payload
                // 4. Commit, if processed successfully.
                let _payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                // consumer.commit_message(&m, CommitMode::Sync).unwrap();
            }
        }
    }
}
