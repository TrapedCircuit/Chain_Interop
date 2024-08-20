use std::collections::HashMap;

use async_trait::async_trait;
use izar_core::{network::IzarNetwork, types::transaction::IzarTransaction};

pub mod aleo;
pub mod eth;
pub mod relayer;

pub const DB_PATH: &str = ".izar-relayer";

#[async_trait]
pub trait Operator<I: IzarNetwork>: Send + Sync {
    async fn execute(&self, tx: &IzarTransaction) -> anyhow::Result<String>;
}

pub type Operators<I> = HashMap<u32, Box<dyn Operator<I>>>;
