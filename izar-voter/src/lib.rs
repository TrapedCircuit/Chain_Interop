use izar_core::types::{cert::Certificate, transaction::IzarTransaction};
use serde::{Deserialize, Serialize};

pub mod aleo;
pub mod eth;
pub mod validators;

pub(crate) const DB_PATH: &str = ".izar-voter";
pub(crate) const UNCONFIRMED_TXS: &str = "uncomfirmed_txs";
pub(crate) const SPEEDUP_TXS: &str = "speedup_txs";

pub trait Validator {
    fn sign(&self, msg: IzarTransaction) -> anyhow::Result<Certificate>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedupRequest {
    lock_tx_hash: String,
    speed_up_hash: String,
}
