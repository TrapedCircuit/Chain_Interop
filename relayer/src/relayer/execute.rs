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
            let from_chain_tx_hash = tx.from_chain_tx_hash.clone();
            let fut = {
                let self_ = self_.clone();
                async move {
                    let op = self_.operators.get(&tx.to_chain_id);
                    let op = op.ok_or(anyhow!("no operator for chain id: {}", tx.to_chain_id))?;
                    let tx_hash = op.execute(&tx).await?;
                    self_.store().finalize().insert(tx.from_chain_tx_hash.clone(), tx.clone())?;
                    Ok::<_, anyhow::Error>(tx_hash)
                }
            };

            tokio::spawn({
                let self_ = self_.clone();
                async move {
                    let status = match fut.await {
                        Ok(tx_hash) => {
                            tracing::info!("tx executed: {}", tx_hash);
                            TransactionStatus::Success(tx_hash)
                        }
                        Err(e) => {
                            tracing::error!("tx failed: {}", e);
                            TransactionStatus::Rejected(e.to_string())
                        }
                    };
                    if let Err(e) = self_.patch_result(from_chain_tx_hash, status).await {
                        tracing::error!("failed to patch result: {}", e);
                    }
                }
            });
        }
    }

    async fn patch_result(&self, from_tx_hash: String, status: TransactionStatus) -> anyhow::Result<()> {
        let req = PatchRequest::new(from_tx_hash, status);
        let resp = self.client.patch(self.sequencer()).json(&req).send().await?;
        tracing::info!("patch result: {:?}", resp);
        Ok(())
    }
}
