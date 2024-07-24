use std::str::FromStr;

use crate::{
    eths,
    network::aleo::AleoNetworkExt,
    serde::{from_payload, to_payload},
    utils::PlaintextCodec,
};
use aleo_rust::{Address, Field, Identifier, Network, Plaintext, ProgramID, ToBytes};
use base64::Engine;
use ethers::types::{H256, U256};
use snarkvm_console::program::FromField;
use snarkvm_utilities::{to_bits_le, ToBits};

use super::{eth::EthAddress, transaction::IzarTransaction};

pub struct AleoTransaction<N: Network> {
    tx_hash: N::TransactionID,
    from_addr: Address<N>,

    to_chain_id: u32,
    to_asset_addr: Field<N>,
    to_addr: Field<N>,

    amount: u128,
    fee: u128,
}

impl<N: Network> AleoTransaction<N> {
    pub fn new(
        tx_hash: N::TransactionID,
        to_chain_id: u32,
        to_asset_addr: Field<N>,
        to_addr: Field<N>,
        from_addr: Address<N>,
        amount: u128,
        fee: u128,
    ) -> Self {
        Self { tx_hash, to_chain_id, to_asset_addr, to_addr, from_addr, amount, fee }
    }
}

impl<N: Network> TryInto<IzarTransaction> for AleoTransaction<N> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<IzarTransaction, Self::Error> {
        let from_asset_addr = ProgramID::<N>::from_str(N::ALEO_PROXY_CONTRACT)?.to_address()?.to_string();
        let AleoTransaction { tx_hash, to_chain_id, to_asset_addr, to_addr, from_addr, amount, fee } = self;

        let (to_asset_addr, to_addr) = match to_chain_id {
            eths!() => (
                EthAddress::<N>::from_field(&to_asset_addr)?.to_string(),
                EthAddress::<N>::from_field(&to_addr)?.to_string(),
            ),
            _ => anyhow::bail!("Unsupported chain id"),
        };
        let nonce = tx_hash.to_bytes_le()?;
        let nonce = U256::from_little_endian(&nonce);

        let payload_data = to_payload(&to_asset_addr, &to_addr, U256::from(amount));
        let payload_zip = base64::engine::general_purpose::STANDARD.encode(payload_data);

        Ok(IzarTransaction {
            priority: Default::default(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs(),
            from_chain_tx_hash: tx_hash.to_string(),
            from_chain_id: N::IZAR_CHAIN_ID,
            from_asset_addr,
            from_addr: from_addr.to_string(),
            to_chain_id,
            to_asset_addr,
            to_addr,
            to_chain_tx_hash: None,

            payload: payload_zip,
            nonce: nonce.to_string(),
            certificates: vec![],
            fee: fee.to_string(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct IzarCrossMsg<N: Network> {
    pub to_chain_id: u32,
    pub to_addr: Field<N>,
    pub to_asset_addr: Field<N>,
    pub token_id: Field<N>,
    pub amount: u128,
    pub fee: u128,
}

impl<N: Network> PlaintextCodec<N> for IzarCrossMsg<N> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Struct(s, _) = p {
            let to_chain_id_ident = Identifier::from_str("to_chain_id")?;
            let to_addr_ident = Identifier::from_str("to_addr")?;
            let to_asset_addr_ident = Identifier::from_str("to_asset_addr")?;
            let token_id_ident = Identifier::from_str("token_id")?;
            let amount_ident = Identifier::from_str("amount")?;
            let fee_ident = Identifier::from_str("fee")?;

            let to_chain_id = PlaintextCodec::<N>::decode(
                s.get(&to_chain_id_ident).ok_or(anyhow::anyhow!("to_chain_id not found"))?,
            )?;
            let to_addr =
                PlaintextCodec::<N>::decode(s.get(&to_addr_ident).ok_or(anyhow::anyhow!("to_addr not found"))?)?;
            let to_asset_addr = PlaintextCodec::<N>::decode(
                s.get(&to_asset_addr_ident).ok_or(anyhow::anyhow!("to_asset_addr not found"))?,
            )?;
            let token_id =
                PlaintextCodec::<N>::decode(s.get(&token_id_ident).ok_or(anyhow::anyhow!("token_id not found"))?)?;
            let amount = PlaintextCodec::<N>::decode(s.get(&amount_ident).ok_or(anyhow::anyhow!("amount not found"))?)?;
            let fee = PlaintextCodec::<N>::decode(s.get(&fee_ident).ok_or(anyhow::anyhow!("fee not found"))?)?;

            Ok(Self { to_chain_id, to_addr, to_asset_addr, token_id, amount, fee })
        } else {
            anyhow::bail!("invalid plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!(
            "{{ to_chain_id: {}u32, to_addr: {}, to_asset_addr: {}, token_id: {} amount: {}u128, fee: {}u128 }}",
            self.to_chain_id, self.to_addr, self.to_asset_addr, self.token_id, self.amount, self.fee
        );
        Plaintext::from_str(&raw_str)
    }
}

pub struct EthRawHash(u128, u128);

impl<N: Network> PlaintextCodec<N> for EthRawHash {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Struct(s, _) = p {
            let h1_ident = Identifier::from_str("h1")?;
            let h2_ident = Identifier::from_str("h2")?;

            let h1 = PlaintextCodec::<N>::decode(s.get(&h1_ident).ok_or(anyhow::anyhow!("hash not found"))?)?;
            let h2 = PlaintextCodec::<N>::decode(s.get(&h2_ident).ok_or(anyhow::anyhow!("nonce not found"))?)?;

            Ok(Self(h1, h2))
        } else {
            anyhow::bail!("invalid plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{{ h1: {}u128, h2: {}u128 }}", self.0, self.1);
        Plaintext::from_str(&raw_str)
    }
}

impl EthRawHash {
    pub fn to_h256(self) -> H256 {
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0..16].copy_from_slice(&self.0.to_le_bytes());
        hash_bytes[16..].copy_from_slice(&self.1.to_le_bytes());

        ethers::types::H256::from_slice(&hash_bytes)
    }
}

pub struct IzarTokenMeta<N: Network> {
    pub from_chain_id: u32,
    pub from_asset_addr: Field<N>,
}

impl<N: Network> PlaintextCodec<N> for IzarTokenMeta<N> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Struct(s, _) = p {
            let from_chain_id_ident = Identifier::from_str("from_chain_id")?;
            let from_asset_addr_ident = Identifier::from_str("from_asset_addr")?;

            let from_chain_id = PlaintextCodec::<N>::decode(
                s.get(&from_chain_id_ident).ok_or(anyhow::anyhow!("from_chain_id not found"))?,
            )?;
            let from_asset_addr = PlaintextCodec::<N>::decode(
                s.get(&from_asset_addr_ident).ok_or(anyhow::anyhow!("from_asset_addr not found"))?,
            )?;

            Ok(Self { from_chain_id, from_asset_addr })
        } else {
            anyhow::bail!("invalid plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str =
            format!("{{ from_chain_id: {}u32, from_asset_addr: {} }}", self.from_chain_id, self.from_asset_addr);
        Plaintext::from_str(&raw_str)
    }
}

#[derive(Clone)]
pub struct IzarRecvMsg<N: Network> {
    pub protocol_addr: Address<N>,
    pub nonce: u128,
    pub to_addr: Address<N>,
    pub from_chain_id: u32,
    pub from_asset_addr: Field<N>,
    pub token_id: Field<N>,
    pub amount: u128,
}

impl<N: Network> TryFrom<&IzarTransaction> for IzarRecvMsg<N> {
    type Error = anyhow::Error;

    fn try_from(value: &IzarTransaction) -> Result<Self, Self::Error> {
        let protocol_addr = ProgramID::<N>::from_str(N::ALEO_PROTOCOL_CONTRACT)?.to_address()?;
        let from_chain_id = value.from_chain_id;
        let nonce = value.nonce.parse::<u128>()?;
        let to_addr = Address::<N>::from_str(&value.to_addr)?;
        let from_asset_addr = N::hash_bhp256(&to_bits_le!(value.from_asset_addr))?;
        let payload = base64::engine::general_purpose::STANDARD.decode(&value.payload)?;
        let amount = from_payload(&payload)?.2.as_u128();

        let token_metadata = IzarTokenMeta::<N> { from_chain_id, from_asset_addr };
        let token_id = N::hash_bhp256(&to_bits_le!(token_metadata.encode()?))?;

        Ok(Self { protocol_addr, nonce, to_addr, from_chain_id, from_asset_addr, token_id, amount })
    }
}

impl<N: Network> PlaintextCodec<N> for IzarRecvMsg<N> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Struct(s, _) = p {
            let protocol_addr_ident = Identifier::from_str("protocol_addr")?;
            let from_chain_id_ident = Identifier::from_str("from_chain_id")?;
            let nonce_ident = Identifier::from_str("nonce")?;
            let to_addr_ident = Identifier::from_str("to_addr")?;
            let from_asset_addr_ident = Identifier::from_str("from_asset_addr")?;
            let amount_ident = Identifier::from_str("amount")?;
            let token_id_ident = Identifier::from_str("token_id")?;

            let protocol_addr = PlaintextCodec::<N>::decode(
                s.get(&protocol_addr_ident).ok_or(anyhow::anyhow!("protocol_addr not found"))?,
            )?;
            let from_chain_id = PlaintextCodec::<N>::decode(
                s.get(&from_chain_id_ident).ok_or(anyhow::anyhow!("from_chain_id not found"))?,
            )?;
            let nonce = PlaintextCodec::<N>::decode(s.get(&nonce_ident).ok_or(anyhow::anyhow!("nonce not found"))?)?;
            let to_addr =
                PlaintextCodec::<N>::decode(s.get(&to_addr_ident).ok_or(anyhow::anyhow!("to_addr not found"))?)?;
            let from_asset_addr = PlaintextCodec::<N>::decode(
                s.get(&from_asset_addr_ident).ok_or(anyhow::anyhow!("to_asset_addr not found"))?,
            )?;
            let amount = PlaintextCodec::<N>::decode(s.get(&amount_ident).ok_or(anyhow::anyhow!("amount not found"))?)?;
            let token_id =
                PlaintextCodec::<N>::decode(s.get(&token_id_ident).ok_or(anyhow::anyhow!("token_id not found"))?)?;

            Ok(Self { protocol_addr, from_chain_id, nonce, to_addr, from_asset_addr, token_id, amount })
        } else {
            anyhow::bail!("invalid plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!(
            "{{ protocol_addr: {}, nonce: {}u128, to_addr: {}, from_chain_id: {}u32,  from_asset_addr: {}, token_id: {}, amount: {}u128 }}",
            self.protocol_addr, self.nonce, self.to_addr, self.from_chain_id, self.from_asset_addr, self.token_id, self.amount
        );

        Plaintext::from_str(&raw_str)
    }
}
