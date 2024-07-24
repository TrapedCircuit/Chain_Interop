use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::cert::Certificate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IzarTransaction {
    pub priority: Priority,
    pub timestamp: u64,

    pub from_chain_tx_hash: String,
    pub from_chain_id: u32,
    pub from_asset_addr: String,
    pub from_addr: String,

    pub to_chain_id: u32,
    pub to_asset_addr: String,
    pub to_addr: String,
    pub to_chain_tx_hash: Option<String>,

    pub payload: String,
    pub nonce: String,
    pub certificates: Vec<Certificate>,

    pub fee: String,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
pub enum Priority {
    High = 0x00,   // 0
    Medium = 0x77, // 119
    Low = 0xFF,    // 255
}

impl Default for Priority {
    fn default() -> Self {
        Self::Low
    }
}

impl IzarTransaction {
    pub fn is_signed(&self) -> bool {
        !self.certificates.is_empty()
    }

    pub fn eth_payload(&self) -> anyhow::Result<Vec<u8>> {
        let payload = base64::engine::general_purpose::STANDARD.decode(&self.payload)?;

        Ok(payload)
    }

    pub fn order_key(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(self.priority as u8);
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(self.from_chain_tx_hash.as_bytes());

        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedupTransaction {
    pub from_chain_tx_hash: String,
    pub speed_up_hash: String,
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use ethers::types::U256;
    use rand::{thread_rng, Rng};

    use crate::{
        serde::to_payload,
        types::{cert::Certificate, transaction::IzarTransaction},
    };

    #[test]
    fn test_tx_bin_serde() {
        let payload = to_payload("to_asset_addr", "to_addr", U256::max_value());
        let payload_base64 = base64::engine::general_purpose::STANDARD.encode(payload);
        let cert = Certificate { signature: "sig1".to_string(), signer: "signer1".to_string() };
        let t = IzarTransaction {
            priority: Default::default(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),

            from_chain_tx_hash: "tx1".to_string(),
            from_chain_id: 2,
            from_asset_addr: "asset1".to_string(),
            from_addr: "addr1".to_string(),

            to_chain_id: 1,
            to_asset_addr: "asset2".to_string(),
            to_addr: "addr2".to_string(),
            to_chain_tx_hash: None,

            payload: payload_base64,
            nonce: "nonce1".to_string(),
            certificates: vec![cert],

            fee: thread_rng().gen::<u128>().to_string(),
        };

        let bin = bincode::serialize(&t).unwrap();
        let t2 = bincode::deserialize(&bin).unwrap();

        assert_eq!(t, t2);

        let json_str = serde_json::to_string(&t).unwrap();
        println!("{}", json_str);
    }
}
