use anyhow::anyhow;
use izar_core::network::IzarNetwork;
use std::sync::Arc;

use crate::relayer::types::{PatchRequest, TransactionStatus};

use super::IzarRelayer;

impl<I: IzarNetwork> IzarRelayer<I> {
    pub async fn execute_handler(self_: Arc<IzarRelayer<I>>) -> anyhow::Result<()> {
        loop {
            let tx = match self_.store().execute().iter().next() {
                Some((_, tx)) => tx.into_owned(),
                None => {
                    tracing::warn!("no execute transaction, sleep 15s");
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    continue;
                }
            };
            let tid = tx.order_key();
            self_.store().execute().remove(&tid)?;

            // check is already finalize
            if self_.store().finalize().get(&tx.from_chain_tx_hash)?.is_some() {
                tracing::warn!("tx already finalized: {}", tx.from_chain_tx_hash);
                self_.store().execute().remove(&tid)?;
                continue;
            }

            // execute tx
            tracing::info!("executing {:?}", tx);
            let op = self_.operators.get(&tx.to_chain_id);
            let fut = async move {
                let op = op.ok_or(anyhow!("no operator for chain id: {}", tx.to_chain_id))?;
                let pending = op.execute(tx).await?;
                Ok::<_, anyhow::Error>(pending)
            };

            match fut.await {
                Ok(pending) => self_.store().pending().insert(tid, pending)?,
                Err(e) => tracing::error!("failed to execute tx: {:?}", e),
            }
        }
    }

    pub fn pending_checker(self_: Arc<IzarRelayer<I>>) {
        loop {
            let tx = match self_.store().pending().iter().next() {
                Some((_, tx)) => tx.into_owned(),
                None => {
                    tracing::warn!("no pending transaction, sleep 60s");
                    std::thread::sleep(std::time::Duration::from_secs(60));
                    continue;
                }
            };
            let tid = tx.order_key();
            let from_chain_tx_hash = tx.from_chain_tx_hash.clone();
            self_.store().pending().remove(&tid).unwrap(); // TODO: handle error

            let izar = self_.clone();
            let fut = async move {
                let op = izar
                    .operators
                    .get(&tx.to_chain_id)
                    .ok_or(anyhow!("no operator for chain id: {}", tx.to_chain_id))?;
                let status = op.pending(tx.clone())?.checking().await;
                match status {
                    TransactionStatus::Success(ref tx_hash) => {
                        tracing::info!("tx {from_chain_tx_hash} finalized: {}", tx_hash);
                        izar.store().finalize().insert(from_chain_tx_hash, tx.clone())?;
                    }
                    TransactionStatus::NotBroadcasted(ref e) => {
                        tracing::error!("tx {from_chain_tx_hash} not broadcasted: {}, re-add executing pipeline", e);
                    }
                    TransactionStatus::Rejected(ref reason) => {
                        tracing::error!("tx {from_chain_tx_hash} rejected: {}", reason)
                    }
                }
                izar.patch_result(tx.from_chain_tx_hash.clone(), status).await?;
                Ok::<_, anyhow::Error>(())
            };

            tokio::spawn(fut); // TODO: handle error? resource control?
        }
    }

    async fn patch_result(&self, from_tx_hash: String, status: TransactionStatus) -> anyhow::Result<()> {
        let req = PatchRequest::new(from_tx_hash, status);
        let resp = self.client.patch(self.sequencer()).json(&req).send().await?;
        tracing::info!("patch result: {:?}", resp);
        Ok(())
    }
}
