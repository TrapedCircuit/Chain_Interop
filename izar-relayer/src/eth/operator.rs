use async_trait::async_trait;
use ethers::{
    contract::abigen,
    types::{Address, Bytes, Signature, H256, U256},
};
use izar_core::{
    network::{eth::EthNetwork, IzarNetwork},
    types::transaction::IzarTransaction,
};
use std::str::FromStr;

use crate::{relayer::types::IzarPendingTransaction, Operator};

use super::connector::EthOperator;

abigen!(Bridge, "./src/eth/abi.json", event_derives(serde::Deserialize, serde::Serialize));

#[async_trait]
impl<I: IzarNetwork, E: EthNetwork> Operator<I> for EthOperator<E> {
    async fn execute(&self, mut tx: IzarTransaction) -> anyhow::Result<IzarTransaction> {
        let sigs = tx
            .certificates
            .iter()
            .map(|c| Signature::from_str(&c.signature).map(|s| s.to_vec()))
            .collect::<Result<Vec<Vec<u8>>, _>>()?
            .concat();
        let sigs = Bytes::from(sigs);
        let from_chain_id = tx.from_chain_id as u16;
        let nonce = U256::from_dec_str(&tx.nonce)?;
        let src_addr_bytes = Bytes::from(tx.from_addr.as_bytes().to_vec());
        let payload = Bytes::from(tx.eth_payload()?);
        let (logic_addr, lock_addr) = (E::LOGIC_CONTRACT.parse::<Address>()?, E::LOCK_CONTRACT.parse::<Address>()?);
        let bridge_call = Bridge::new(logic_addr, self.client.clone());

        // need support eip1159
        let tx_hash = bridge_call
            .receive_payload(from_chain_id, nonce, src_addr_bytes, lock_addr, payload, sigs, U256::from(E::GAS_LIMIT))
            .gas(E::GAS_LIMIT)
            .send()
            .await?
            .tx_hash();

        tx.to_chain_tx_hash = Some(E::format_str(tx_hash));
        Ok(tx)
    }

    fn pending(&self, tx: IzarTransaction) -> anyhow::Result<IzarPendingTransaction<I>> {
        let tx_hash = H256::from_str(&tx.to_chain_tx_hash.expect("empty to chain tx hash"))?;
        Ok(IzarPendingTransaction::eth(tx_hash, self.client.clone(), E::CONFIRMATIONS))
    }
}
