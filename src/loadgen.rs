use serde::Serialize;
use std::time::Duration;

use async_channel::TrySendError;
use tokio::time::interval;
use tracing::{info, warn};

use crate::{
    client::{self, Dispatcher, DispatcherGenerator},
    input::InputGenerator,
    writer::Writer,
};

pub enum Msg {
    Sleep(Duration),
}

/// Tries to generate a request every interval milliseconds for a total number of requests.
/// If it would block trying to spawn the request it will create a new client.
pub async fn generate_load<
    D: DispatcherGenerator,
    I: InputGenerator<Input = <D::Dispatcher as Dispatcher>::Input>,
    W: Writer<<D::Dispatcher as Dispatcher>::Output> + 'static,
>(
    // TODO: move to options struct
    rate: u64,
    initial_clients: u64,
    total: u64,
    max_clients: Option<u32>,
    mut input_generator: I,
    mut dispatcher_generator: D,
    writer: &mut W,
) where
    <D::Dispatcher as Dispatcher>::Output: Serialize + Default,
{
    let (input_sender, input_receiver) = async_channel::bounded(1);

    let nanos_in_second = 1_000_000_000;
    let interval_nanos = nanos_in_second / rate;

    let mut ticker = interval(Duration::from_nanos(interval_nanos));

    let mut client_counter = 0;
    let mut i = 0;

    let mut tasks = Vec::with_capacity(initial_clients as usize);

    for _ in 0..initial_clients {
        client_counter += 1;
        let receiver = input_receiver.clone();

        let dispatcher = dispatcher_generator.generate();
        let task =
            tokio::spawn(async move { client::run(receiver, client_counter, dispatcher).await });
        tasks.push(task);
    }

    while let Some(input) = input_generator.next() {
        if i % rate == 0 {
            info!(done = i, total = total, "Progressing");
        }
        match input_sender.try_send(input) {
            Ok(()) => {
                // sent successfully, there must have been an available client
            }
            // TODO: maybe preallocate clients, or always keep a few spare
            Err(TrySendError::Full(input)) => {
                // wasn't available so create a new client to service the request
                let generate_new_client = if let Some(max_clients) = max_clients {
                    client_counter < max_clients
                } else {
                    true
                };
                if generate_new_client {
                    client_counter += 1;
                    let receiver = input_receiver.clone();
                    let dispatcher = dispatcher_generator.generate();
                    let task = tokio::spawn(async move {
                        client::run(receiver, client_counter, dispatcher).await
                    });
                    tasks.push(task);
                }
                input_sender.send(input).await.unwrap();
            }
            Err(TrySendError::Closed(_value)) => {
                // nothing else to do but stop the loop
                break;
            }
        }

        ticker.tick().await;

        i += 1;
        if i == total {
            break;
        }
    }

    info!("Closing load sender");
    input_sender.close();
    info!("Closing input generator");
    input_generator.close();

    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(outputs) => {
                for output in outputs {
                    writer.write(output).await;
                }
            }
            Err(error) => {
                warn!(%error, task=i, "Failed to join task");
            }
        }
    }

    info!(clients=%client_counter, "Finished generating load");
}
