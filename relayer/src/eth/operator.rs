use async_trait::async_trait;
use ethers::{
    contract::abigen,
    types::{Address, Bytes, Signature, U256},
};
use izar_core::{
    network::{eth::EthNetwork, IzarNetwork},
    types::transaction::IzarTransaction,
};
use std::str::FromStr;

use crate::Operator;

use super::connector::EthOperator;

abigen!(Bridge, "./src/eth/abi.json", event_derives(serde::Deserialize, serde::Serialize));

#[async_trait]
impl<I: IzarNetwork, E: EthNetwork> Operator<I> for EthOperator<E> {
    async fn execute(&self, tx: &IzarTransaction) -> anyhow::Result<String> {
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
        let receipt = bridge_call
            .receive_payload(from_chain_id, nonce, src_addr_bytes, lock_addr, payload, sigs, U256::from(E::GAS_LIMIT))
            .gas(E::GAS_LIMIT)
            .send()
            .await?
            .await?
            .ok_or(anyhow::anyhow!("tx failed"))?;

        // return tx_hash
        Ok(E::format_str(receipt.transaction_hash))
    }
}
