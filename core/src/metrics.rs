use std::{collections::HashMap, time::Duration};

use metrics_exporter_prometheus::PrometheusBuilder;

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub dest: String,
    pub interval: Duration,
    pub labels: HashMap<String, String>,
}

impl MetricsConfig {
    pub fn new(dest: String, interval: Duration) -> Self {
        Self { dest, interval, labels: HashMap::new() }
    }

    pub fn with_labels(&mut self, labels: Vec<(&str, &str)>) {
        for (k, v) in labels {
            self.labels.insert(k.to_string(), v.to_string());
        }
    }

    pub fn init(&self) -> anyhow::Result<()> {
        let mut dest = format!("{}/metrics/job/relayer", self.dest);
        for (k, v) in &self.labels {
            dest.push_str(&format!("/{}/{}", k, v));
        }
        metrics_init(&dest, self.interval)
    }
}

pub fn metrics_init(dest: &str, interval: Duration) -> anyhow::Result<()> {
    PrometheusBuilder::new()
        .with_push_gateway(dest, interval, None, None)
        .map_err(|e| anyhow::anyhow!("metrics init error: {}", e))?
        .install()
        .map_err(|e| anyhow::anyhow!("metrics init error: {}", e))
}

pub mod relayer {
    pub const RELAYER_BALANCE: &str = "relayer_balance";
    pub const RELAYER_BLOCK_HEIGHT: &str = "relayer_block_height";
}
