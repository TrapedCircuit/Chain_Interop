use std::collections::HashMap;

use izar_core::{
    db::{map::DBMap, RocksDB},
    types::transaction::{IzarTransaction, SpeedupTransaction},
};
use tracing::{error_span, Instrument};

use crate::{Validator, DB_PATH, SPEEDUP_TXS, UNCONFIRMED_TXS};

pub struct IzarValidators {
    connectors: HashMap<u32, Box<dyn Validator>>,
    unconfirmd_txs: DBMap<String, IzarTransaction>,
    speedup_txs: DBMap<String, String>,
    dest: String,
    client: ureq::Agent,
}

impl IzarValidators {
    pub fn new(dest: String) -> Self {
        Self {
            connectors: HashMap::new(),
            unconfirmd_txs: RocksDB::open_map(DB_PATH, UNCONFIRMED_TXS).expect("Failed to open uncomfirmed txs db"),
            speedup_txs: RocksDB::open_map(DB_PATH, SPEEDUP_TXS).expect("Failed to open speedup txs db"),
            dest,
            client: ureq::agent(),
        }
    }

    pub fn insert_connector(&mut self, chain_id: u32, connector: Box<dyn Validator>) {
        self.connectors.insert(chain_id, connector);
    }

    fn sign_tx(&self, tx: IzarTransaction) -> anyhow::Result<izar_core::types::cert::Certificate> {
        let connector = self
            .connectors
            .get(&tx.to_chain_id)
            .ok_or_else(|| anyhow::anyhow!("chain id {:?} validator not found", tx.to_chain_id))?;
        connector.sign(tx)
    }

    async fn handle_txs(&self) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/BridgeTx", self.dest);
        let txs = self.unconfirmd_txs.get_all()?;
        for (_, mut tx) in txs {
            self.unconfirmd_txs.remove(&tx.from_chain_tx_hash)?;
            let cert = self.sign_tx(tx.clone())?;
            tx.certificates.push(cert);

            let resp = self.client.post(&url).send_json(&tx)?;
            match resp.status() {
                400..=499 => tracing::error!("submit response error {}", resp.into_string()?),
                200..=299 => tracing::info!("submit sigs success: {:?}", tx),
                _ => {
                    tracing::error!("unimplemented status code {}", resp.status());
                    self.unconfirmd_txs.insert(tx.from_chain_tx_hash.clone(), tx)?;
                }
            }
        }
        Ok(())
    }

    async fn handle_speedup_txs(&self) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/BridgeTxSpeedUp", self.dest);
        let txs = self.speedup_txs.get_all()?;
        for tx in txs {
            let (lock_tx_hash, speed_up_hash) = (tx.0, tx.1);
            self.speedup_txs.remove(&speed_up_hash)?;
            let req =
                SpeedupTransaction { from_chain_tx_hash: lock_tx_hash.clone(), speed_up_hash: speed_up_hash.clone() };
            let resp = self.client.post(&url).send_json(&req)?;
            match resp.status() {
                400..=499 => tracing::error!("submit response error {}", resp.into_string()?),
                200..=299 => tracing::info!("submit speedup tx success: {:?}", req),
                _ => {
                    tracing::error!("unimplemented status code {}", resp.status());
                    self.speedup_txs.insert(speed_up_hash, lock_tx_hash)?;
                }
            }
        }

        Ok(())
    }

    pub async fn initial(self) {
        if self.connectors.is_empty() {
            panic!("no connector found");
        }

        loop {
            if let Err(e) = self.handle_txs().instrument(error_span!("TXS")).await {
                tracing::error!("handle txs error: {:?}", e);
            }

            if let Err(e) = self.handle_speedup_txs().instrument(error_span!("SPEEDUP")).await {
                tracing::error!("handle speedup txs error: {:?}", e);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        }
    }
}
