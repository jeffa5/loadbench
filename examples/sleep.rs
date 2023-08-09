use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::metadata::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use loadbench::{
    client::{Dispatcher, DispatcherGenerator},
    input::InputGenerator,
    loadgen,
    writer::StatsWriter,
    Output,
};

pub struct SleepInputGenerator {
    pub milliseconds: f64,
}

impl InputGenerator for SleepInputGenerator {
    type Input = Duration;

    fn close(self) {}

    fn next(&mut self) -> Option<Self::Input> {
        Some(Duration::from_nanos(
            (self.milliseconds * 1_000_000.) as u64,
        ))
    }
}

struct SleepDispatcherGenerator {}

impl DispatcherGenerator for SleepDispatcherGenerator {
    type Dispatcher = SleepDispatcher;

    fn generate(&mut self) -> Self::Dispatcher {
        SleepDispatcher {}
    }
}

#[derive(Clone)]
pub struct SleepDispatcher {}

#[async_trait::async_trait]
impl Dispatcher for SleepDispatcher {
    type Input = Duration;
    type Output = SleepOutput;

    async fn execute_scenario(
        &mut self,
        client: u32,
        iteration: u32,
        duration: Self::Input,
    ) -> Vec<Output<Self::Output>> {
        let mut output = Output::start(client, iteration);
        tokio::time::sleep(duration).await;
        output.stop();
        vec![output]
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SleepOutput {}

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "100")]
    rate: u64,
    #[clap(long, default_value = "1000")]
    total: u64,

    #[clap(long, default_value = "0")]
    initial_clients: u64,
    #[clap(long)]
    max_clients: Option<u32>,

    #[clap(long, default_value = "100")]
    sleep_ms: f64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let sleep_input = SleepInputGenerator {
        milliseconds: args.sleep_ms,
    };

    let sleep_dispatcher = SleepDispatcherGenerator {};
    let mut writer = StatsWriter::default();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    loadgen::generate_load(
        args.rate,
        args.initial_clients,
        args.total,
        args.max_clients,
        sleep_input,
        sleep_dispatcher,
        &mut writer,
    )
    .await;

    writer.summary();
}
