use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output<D>(pub OutputCore, pub D);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputCore {
    pub start_ns: i64,
    pub end_ns: i64,
    pub error: Option<String>,
    pub client: u32,
    pub iteration: u32,
}

impl<D: Default> Output<D> {
    pub fn start(client: u32, iteration: u32) -> Self {
        let now = chrono::Utc::now();
        Self(
            OutputCore {
                start_ns: now.timestamp_nanos(),
                end_ns: now.timestamp_nanos(),
                error: None,
                client,
                iteration,
            },
            D::default(),
        )
    }
}

impl<D> Output<D> {
    pub fn stop(&mut self) {
        self.0.end_ns = chrono::Utc::now().timestamp_nanos();
    }

    pub fn error(&mut self, error: String) {
        self.0.error = Some(error);
        self.0.end_ns = chrono::Utc::now().timestamp_nanos();
    }

    pub fn is_error(&self) -> bool {
        self.0.error.is_some()
    }

    pub fn data_mut(&mut self) -> &mut D {
        &mut self.1
    }
}

impl<D> Drop for Output<D> {
    fn drop(&mut self) {
        if self.0.end_ns == self.0.start_ns {
            self.stop()
        }
    }
}
