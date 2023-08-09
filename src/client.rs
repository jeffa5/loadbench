use serde::Serialize;
use tracing::{debug, trace};

use crate::output::Output;
use async_trait::async_trait;

pub trait DispatcherGenerator {
    type Dispatcher: Dispatcher;
    fn generate(&mut self) -> Self::Dispatcher;
}

#[async_trait]
pub trait Dispatcher: Send + 'static {
    type Input: Send;
    type Output: Send + Default;
    async fn execute_scenario(
        &mut self,
        client: u32,
        iteration: u32,
        request: Self::Input,
    ) -> Vec<Output<Self::Output>>;
}

pub async fn run<D: Dispatcher>(
    receiver: async_channel::Receiver<D::Input>,
    client: u32,
    mut dispatcher: D,
) -> Vec<Output<D::Output>>
where
    D::Output: Serialize + Default,
{
    let mut all_outputs = Vec::new();
    let mut iteration = 0;
    while let Ok(input) = receiver.recv().await {
        let mut outputs = dispatcher.execute_scenario(client, iteration, input).await;
        all_outputs.append(&mut outputs);

        iteration += 1;
        trace!(%client, %iteration, "Client finished iteration");
    }

    debug!(%client, "Client finished dispatching");
    all_outputs
}
