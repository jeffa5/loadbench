use async_trait::async_trait;
use serde::Serialize;

use crate::Output;

#[async_trait]
pub trait Writer<O> {
    async fn write(&mut self, output: Output<O>);
}

/// Do nothing with the outputs.
pub struct NoOpWriter;

#[async_trait]
impl<O: Send + 'static> Writer<O> for NoOpWriter {
    async fn write(&mut self, _output: Output<O>) {}
}

#[derive(Default, Debug)]
pub struct StatsWriter {
    error_count: u64,
    success_count: u64,
    latency_ns: Vec<i64>,
    start_ns: i64,
    end_ns: i64,
}

#[async_trait]
impl<O: Send + 'static> Writer<O> for StatsWriter {
    async fn write(&mut self, output: Output<O>) {
        if output.is_error() {
            self.error_count += 1;
        } else {
            self.success_count += 1;
        }

        let latency_ns = output.0.end_ns - output.0.start_ns;
        self.latency_ns.push(latency_ns);

        if self.start_ns == 0 {
            self.start_ns = output.0.start_ns;
        }
        self.start_ns = std::cmp::min(self.start_ns, output.0.start_ns);
        self.end_ns = std::cmp::max(self.end_ns, output.0.end_ns);
    }
}

impl StatsWriter {
    pub fn summary(&self) {
        let total = self.success_count + self.error_count;
        println!("     Total requests: {}", total);
        println!("Successful requests: {}", self.success_count);
        println!(" Erroneous requests: {}", self.error_count);

        let duration_ns = self.end_ns - self.start_ns;
        let duration_s = duration_ns as f64 / 1_000_000_000.;
        println!("Total time (seconds): {}", duration_s);
        let tp = total as f64 / duration_s;
        let tp_success = self.success_count as f64 / duration_s;
        let tp_error = self.error_count as f64 / duration_s;

        println!("     Total throughput (req/s): {}", tp);
        println!("Successful throughput (req/s): {}", tp_success);
        println!(" Erroneous throughput (req/s): {}", tp_error);

        let mut latencies = self.latency_ns.clone();
        latencies.sort_unstable();

        let percentile = |latencies: &[i64], percentile: f64| {
            let index = (latencies.len() - 1) as f64 * percentile;
            latencies[index as usize]
        };

        println!("  0% latency (ns): {}", latencies.first().unwrap());
        println!(" 50% latency (ns): {}", percentile(&latencies, 0.5));
        println!(" 90% latency (ns): {}", percentile(&latencies, 0.9));
        println!(" 99% latency (ns): {}", percentile(&latencies, 0.99));
        println!("100% latency (ns): {}", latencies.last().unwrap());
    }
}

pub struct CsvWriter<W: std::io::Write> {
    pub writer: csv::Writer<W>,
}

#[async_trait]
impl<O: Serialize + Send + 'static, W: std::io::Write + Send> Writer<O> for CsvWriter<W> {
    async fn write(&mut self, output: Output<O>) {
        self.writer.serialize(output).unwrap();
    }
}
