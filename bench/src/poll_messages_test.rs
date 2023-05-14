use crate::args::Args;
use crate::test_client::create_connected_client;
use sdk::client::ConnectedClient;
use sdk::error::Error;
use shared::messages::poll_messages::{Format, PollMessages};
use std::collections::HashMap;
use tokio::task;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::info;

pub async fn init_poll_messages(args: &Args) -> Result<Vec<JoinHandle<()>>, Error> {
    info!("Creating {} client(s)...", args.clients_count);
    let mut futures = Vec::with_capacity(args.clients_count as usize);
    let messages_per_batch = args.messages_per_batch;
    let message_batches = args.message_batches;
    for i in 0..args.clients_count {
        let client_id = i + 1;
        let client = create_connected_client(&args.server_address, &args.server_name).await?;
        let future = task::spawn(async move {
            info!("Executing the test on client #{}...", client_id);
            let stream_id: u32 = 10000 + client_id;
            let topic_id: u32 = 1;
            let partition_id: u32 = 1;
            let result = execute_poll_messages(
                &client,
                client_id,
                stream_id,
                topic_id,
                partition_id,
                messages_per_batch,
                message_batches,
            )
            .await;
            match result {
                Ok(_) => info!("Executed poll messages the test on client #{}.", client_id),
                Err(error) => info!("Error on client #{}: {:?}", client_id, error),
            }
        });
        futures.push(future);
    }
    info!("Created {} client(s).", args.clients_count);

    Ok(futures)
}

async fn execute_poll_messages(
    client: &ConnectedClient,
    client_id: u32,
    stream_id: u32,
    topic_id: u32,
    partition_id: u32,
    messages_per_batch: u32,
    batches_count: u32,
) -> Result<(), Error> {
    let total_messages = messages_per_batch * batches_count;
    info!("client #{} → preparing the test messages...", client_id);
    let mut message_batches: HashMap<u32, PollMessages> = HashMap::new();

    for i in 0..batches_count {
        let offset = (i * messages_per_batch) as u64;
        let command = PollMessages {
            consumer_id: client_id,
            stream_id,
            topic_id,
            partition_id,
            kind: 0,
            value: offset,
            count: messages_per_batch,
            auto_commit: false,
            format: Format::Binary,
        };

        message_batches.insert(i, command);
    }

    info!(
        "client #{} → polling {} messages in {} batches of {} messages...",
        client_id, total_messages, batches_count, messages_per_batch
    );

    let start = Instant::now();

    for i in 0..batches_count {
        let command = message_batches.get(&i).unwrap();
        client.poll_messages(command).await?;
    }

    let duration = start.elapsed();

    info!(
        "client #{} → polled {} messages in {} batches of {} messages in {} ms",
        client_id,
        total_messages,
        batches_count,
        messages_per_batch,
        duration.as_millis(),
    );

    Ok(())
}
