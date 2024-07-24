use std::sync::Arc;

use aleo_rust::Network;
use aleo_rust::Transaction;
use axum::response::IntoResponse;
use backon::ExponentialBuilder;
use backon::Retryable;
use ethers::{
    prelude::SignerMiddleware,
    providers::{Http, Middleware, PendingTransaction, Provider},
    signers::LocalWallet,
    types::H256,
};
use izar_core::network::IzarNetwork;
use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Success(String),
    NotBroadcasted(String),
    Rejected(String),
}

impl Serialize for TransactionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TransactionStatus::Success(tx_hash) => {
                let mut s = serializer.serialize_struct("status", 2)?;
                s.serialize_field("code", &1)?;
                s.serialize_field("result", tx_hash)?;
                s.end()
            }
            TransactionStatus::NotBroadcasted(e) => {
                let mut s = serializer.serialize_struct("status", 2)?;
                s.serialize_field("code", &2)?;
                s.serialize_field("result", e)?;
                s.end()
            }
            TransactionStatus::Rejected(e) => {
                let mut s = serializer.serialize_struct("status", 2)?;
                s.serialize_field("code", &3)?;
                s.serialize_field("result", e)?;
                s.end()
            }
        }
    }
}

impl IntoResponse for TransactionStatus {
    fn into_response(self) -> axum::response::Response {
        axum::response::Json(self).into_response()
    }
}

#[derive(Debug, Clone)]
pub enum IzarPendingTransaction<I: IzarNetwork> {
    Aleo(AleoPendingTx<I::Aleo>),
    Eth(EthPendingTx),
}

impl<I: IzarNetwork> IzarPendingTransaction<I> {
    pub fn aleo(tx_hash: <I::Aleo as Network>::TransactionID, client: reqwest::Client, base_url: &str) -> Self {
        Self::Aleo(AleoPendingTx { tx_hash, client, base_url: base_url.to_string() })
    }

    pub fn eth(
        tx_hash: H256,
        client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
        confirmations: usize,
    ) -> Self {
        Self::Eth(EthPendingTx { tx_hash, client, confirmations })
    }

    pub async fn checking(self) -> TransactionStatus {
        match self {
            Self::Aleo(a) => {
                let AleoPendingTx { tx_hash, client, base_url } = a;
                let url = format!("{}/testnet3/transaction/{}", base_url, tx_hash);
                let fut = || async {
                    let tx = client.get(&url).send().await?.json::<Transaction<I::Aleo>>().await?;
                    Ok::<_, anyhow::Error>(tx)
                };
                let receipt = fut.retry(&ExponentialBuilder::default().with_max_times(10));
                match receipt.await {
                    Ok(receipt) if receipt.is_fee() => {
                        TransactionStatus::Rejected("tx was rejected by onchain finalize".to_string())
                    }
                    Ok(_) => TransactionStatus::Success(tx_hash.to_string()),
                    Err(e) => TransactionStatus::NotBroadcasted(format!("tx not broadcasted: {}", e)),
                }
            }
            Self::Eth(e) => {
                let pending_tx = PendingTransaction::new(e.tx_hash, e.client.provider()).confirmations(e.confirmations);
                match pending_tx.await {
                    Ok(r) => {
                        TransactionStatus::Success(format!("{:#020x}", r.expect("must have receipt").transaction_hash))
                    }
                    Err(e) => TransactionStatus::Rejected(format!("tx execute error: {}", e)),
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct AleoPendingTx<N: Network> {
    pub tx_hash: N::TransactionID,
    pub base_url: String,
    pub client: reqwest::Client,
}

impl<N: Network> std::fmt::Debug for AleoPendingTx<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AleoPendingTx").field("tx_hash", &self.tx_hash).finish()
    }
}

#[derive(Clone)]
pub struct EthPendingTx {
    pub tx_hash: H256,
    pub confirmations: usize,
    pub client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl std::fmt::Debug for EthPendingTx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthPendingTx").field("tx_hash", &self.tx_hash).finish()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchRequest {
    from_chain_tx_hash: String,
    status: TransactionStatus,
}

impl PatchRequest {
    pub fn new(from_chain_tx_hash: String, status: TransactionStatus) -> Self {
        Self { from_chain_tx_hash, status }
    }
}
