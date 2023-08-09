use clap::Parser;
use tracing::metadata::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use loadbench::{
    client::{Dispatcher, DispatcherGenerator},
    generate_load,
    input::InputGenerator,
    output_sink::StatsOutputSink,
};

pub struct NoopInputGenerator;

impl InputGenerator for NoopInputGenerator {
    type Input = ();

    fn close(self) {}

    fn next(&mut self) -> Option<Self::Input> {
        Some(())
    }
}

struct NoopDispatcherGenerator;

impl DispatcherGenerator for NoopDispatcherGenerator {
    type Dispatcher = NoopDispatcher;

    fn generate(&mut self) -> Self::Dispatcher {
        NoopDispatcher {}
    }
}

#[derive(Clone)]
pub struct NoopDispatcher;

#[async_trait::async_trait]
impl Dispatcher for NoopDispatcher {
    type Input = ();
    type Output = ();

    async fn execute(&mut self, _: Self::Input) -> Result<Self::Output, String> {
        Ok(())
    }
}

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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let input = NoopInputGenerator {};

    let dispatcher = NoopDispatcherGenerator {};
    let mut writer = StatsOutputSink::default();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    generate_load(
        args.rate,
        args.initial_clients,
        args.total,
        args.max_clients,
        input,
        dispatcher,
        &mut writer,
    )
    .await;

    writer.summary();
}
