use std::collections::HashMap;

use async_trait::async_trait;
use izar_core::{network::IzarNetwork, types::transaction::IzarTransaction};
use relayer::types::IzarPendingTransaction;

pub mod aleo;
pub mod eth;
pub mod relayer;

pub const DB_PATH: &str = ".izar-relayer";

#[async_trait]
pub trait Operator<I: IzarNetwork>: Send + Sync {
    async fn execute(&self, tx: IzarTransaction) -> anyhow::Result<IzarTransaction>;
    fn pending(&self, tx: IzarTransaction) -> anyhow::Result<IzarPendingTransaction<I>>;
}

pub type Operators<I> = HashMap<u32, Box<dyn Operator<I>>>;
