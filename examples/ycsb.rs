use async_trait::async_trait;
use clap::Parser;
use loadbench::client::{Dispatcher, DispatcherGenerator};
use loadbench::loadgen;
use loadbench::{input::InputGenerator, writer::StatsWriter};
use rand::SeedableRng;
use rand::{distributions::Alphanumeric, rngs::StdRng, Rng};
use rand_distr::{Distribution, WeightedAliasIndex, Zipf};
use tracing::metadata::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Generate inputs for the YCSB workloads.
pub struct YcsbInputGenerator {
    pub read_weight: u32,
    pub scan_weight: u32,
    pub insert_weight: u32,
    pub update_weight: u32,
    pub read_all_fields: bool,
    pub fields_per_record: u32,
    pub field_value_length: usize,
    pub operation_rng: StdRng,
    pub max_record_index: u32,
    pub request_distribution: RequestDistribution,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum RequestDistribution {
    /// Uniformly over the existing keys.
    Uniform,
    /// Weighted toward one end.
    Zipfian,
    /// The last one available.
    Latest,
}

impl YcsbInputGenerator {
    pub fn new_record_key(&mut self) -> String {
        // TODO: may not want incremental inserts
        self.max_record_index += 1;
        format!("user{:08}", self.max_record_index)
    }

    pub fn existing_record_key(&mut self) -> String {
        let index = match self.request_distribution {
            RequestDistribution::Zipfian => {
                let s: f64 = self
                    .operation_rng
                    .sample(Zipf::new(self.max_record_index as u64, 1.5).unwrap());
                s.floor() as u32
            }
            RequestDistribution::Uniform => self.operation_rng.gen_range(0..=self.max_record_index),
            RequestDistribution::Latest => self.max_record_index,
        };
        format!("user{:08}", index)
    }

    pub fn field_key(i: u32) -> String {
        format!("field{i}")
    }

    pub fn field_value(&mut self) -> String {
        (&mut self.operation_rng)
            .sample_iter(&Alphanumeric)
            .take(self.field_value_length)
            .map(char::from)
            .collect()
    }
}

#[derive(Debug)]
pub enum YcsbInput {
    /// Insert a new record.
    Insert {
        record_key: String,
        fields: Vec<(String, String)>,
    },
    /// Update a record by replacing the value of one field.
    Update {
        record_key: String,
        field_key: String,
        field_value: String,
    },
    /// Read a single, randomly chosen field from the record.
    ReadSingle {
        record_key: String,
        field_key: String,
    },
    /// Read all fields from a record.
    ReadAll { record_key: String },
    /// Scan records in order, starting at a randomly chosen key
    Scan { start_key: String, scan_length: u32 },
}

impl InputGenerator for YcsbInputGenerator {
    type Input = YcsbInput;

    fn close(self) {}

    fn next(&mut self) -> Option<Self::Input> {
        let weights = [
            self.read_weight,
            self.scan_weight,
            self.insert_weight,
            self.update_weight,
        ];
        let dist = WeightedAliasIndex::new(weights.to_vec()).unwrap();
        let weight_index = dist.sample(&mut self.operation_rng);
        let input = match weight_index {
            // read single
            0 => YcsbInput::ReadSingle {
                record_key: self.existing_record_key(),
                field_key: "field0".to_owned(),
            },
            // read all
            1 => YcsbInput::ReadAll {
                record_key: self.existing_record_key(),
            },
            // insert
            2 => YcsbInput::Insert {
                record_key: self.new_record_key(),
                fields: (0..self.fields_per_record)
                    .into_iter()
                    .map(|i| (Self::field_key(i), self.field_value()))
                    .collect(),
            },
            // update
            3 => YcsbInput::Update {
                record_key: self.existing_record_key(),
                field_key: "field0".to_owned(),
                field_value: random_string(self.field_value_length),
            },
            i => {
                println!("got weight index {i}, but there was no input type to match");
                return None;
            }
        };
        // println!("generated ycsb input {:?}", input);
        Some(input)
    }
}

fn random_string(len: usize) -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
    s
}

struct YcsbDispatcherGenerator {}

impl DispatcherGenerator for YcsbDispatcherGenerator {
    type Dispatcher = YcsbDispatcher;

    fn generate(&mut self) -> Self::Dispatcher {
        YcsbDispatcher {}
    }
}

struct YcsbDispatcher {}

#[async_trait]
impl Dispatcher for YcsbDispatcher {
    type Input = YcsbInput;

    type Output = ();

    async fn execute(&mut self, _request: Self::Input) -> Result<Self::Output, String> {
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

    #[clap(long, default_value = "1")]
    read_weight: u32,
    #[clap(long, default_value = "1")]
    scan_weight: u32,
    #[clap(long, default_value = "1")]
    insert_weight: u32,
    #[clap(long, default_value = "1")]
    update_weight: u32,
    #[clap(long)]
    read_all_fields: bool,
    #[clap(long, default_value = "1")]
    fields_per_record: u32,
    #[clap(long, default_value = "1")]
    field_value_length: usize,
    #[clap(long, default_value = "1")]
    max_record_index: u32,
    #[clap(long, default_value = "1")]
    request_distribution: RequestDistribution,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let sleep_input = YcsbInputGenerator {
        read_weight: args.read_weight,
        scan_weight: args.scan_weight,
        insert_weight: args.insert_weight,
        update_weight: args.update_weight,
        read_all_fields: args.read_all_fields,
        fields_per_record: args.fields_per_record,
        field_value_length: args.field_value_length,
        operation_rng: StdRng::from_entropy(),
        max_record_index: args.max_record_index,
        request_distribution: args.request_distribution,
    };

    let sleep_dispatcher = YcsbDispatcherGenerator {};
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
