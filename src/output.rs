use serde::{Deserialize, Serialize};

/// The output of an execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output<D> {
    /// The core output metrics.
    #[serde(flatten)]
    pub core: OutputCore,
    /// The custom data recorded.
    #[serde(flatten)]
    pub custom: D,
}

/// Core data captured by loadbench.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputCore {
    /// Start time of this execution.
    pub start_ns: i64,
    /// End time of this execution.
    pub end_ns: i64,
    /// An error that may have occurred.
    pub error: Option<String>,
    /// The client that ran the execution.
    pub client: u32,
    /// The iteration of the client that this execution became.
    pub iteration: u32,
}

impl<D: Default> Output<D> {
    pub fn start(client: u32, iteration: u32) -> Self {
        let now = chrono::Utc::now();
        Self {
            core: OutputCore {
                client,
                iteration,
                start_ns: now.timestamp_nanos(),
                end_ns: now.timestamp_nanos(),
                error: None,
            },
            custom: D::default(),
        }
    }
}

impl<D> Output<D> {
    pub fn stop(&mut self) {
        self.core.end_ns = chrono::Utc::now().timestamp_nanos();
    }

    pub fn error(&mut self, error: String) {
        self.core.error = Some(error);
        self.core.end_ns = chrono::Utc::now().timestamp_nanos();
    }

    pub fn is_error(&self) -> bool {
        self.core.error.is_some()
    }

    pub fn data_mut(&mut self) -> &mut D {
        &mut self.custom
    }
}

impl<D> Drop for Output<D> {
    fn drop(&mut self) {
        if self.core.end_ns == self.core.start_ns {
            self.stop()
        }
    }
}
