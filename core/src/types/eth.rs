use std::str::FromStr;

use crate::{network::eth::EthNetwork, serde::ZeroCopyWriter};
use aleo_rust::{Field, Network};
use base64::Engine;
use ethers::{
    abi,
    contract::abigen,
    prelude::EthLogDecode,
    types::{Address, Bytes, H256, U256},
};
use snarkvm_console::program::{FromField, ToField};
use snarkvm_utilities::{FromBytes, ToBytes};

use super::transaction::IzarTransaction;

#[derive(Debug, Clone)]
pub struct EthTransaction<E: EthNetwork> {
    tx_hash: H256,
    from_addr: Address,
    from_asset_addr: Address,
    to_chain_id: u32,
    fee: U256,
    nonce: U256,
    payload: Bytes,

    phantom: std::marker::PhantomData<E>,
}

impl<E: EthNetwork> EthTransaction<E> {
    pub fn from_logs(fee_log: EventFee, payload_log: EventPayload, tx_hash: H256) -> anyhow::Result<EthTransaction<E>> {
        let EventFee { token_address, sender, to_chain_id, to_address: _, amount: _, fee } = fee_log;
        let EventPayload { sender: _, nonce, dst_chain_id: _, destination: _, payload } = payload_log;

        Ok(EthTransaction {
            tx_hash,
            from_addr: sender,
            from_asset_addr: token_address,
            to_chain_id: to_chain_id.as_u32(),
            fee,
            payload,
            nonce,
            phantom: std::marker::PhantomData,
        })
    }
}

impl<E: EthNetwork> TryInto<IzarTransaction> for EthTransaction<E> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<IzarTransaction, Self::Error> {
        let EthTransaction { tx_hash, from_addr, from_asset_addr, to_chain_id, fee, payload, nonce, .. } = self;

        let mut deser = ZeroCopyWriter::from(payload.to_vec());
        let to_asset_addr = String::from_utf8(deser.read_next_bytes())?;
        let to_addr = String::from_utf8(deser.read_next_bytes())?;
        let _amount = deser.read_u256();

        let payload_zip = base64::engine::general_purpose::STANDARD.encode(&payload);

        Ok(IzarTransaction {
            priority: Default::default(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs(),
            from_chain_tx_hash: E::format_str(tx_hash),
            from_chain_id: E::IZAR_CHAIN_ID,
            from_asset_addr: E::format_str(from_asset_addr),
            from_addr: E::format_str(from_addr),

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

#[derive(Debug, Clone)]
pub struct EventFee {
    pub token_address: Address,
    pub sender: Address,
    pub to_chain_id: U256,
    pub to_address: Address,
    pub amount: U256,
    pub fee: U256,
}

impl EthLogDecode for EventFee {
    fn decode_log(log: &abi::RawLog) -> std::result::Result<Self, abi::Error>
    where
        Self: Sized,
    {
        let token_address = Address::from(log.topics[1]);
        let sender = Address::from(log.topics[2]);
        let to_chain_id = U256::from_big_endian(&log.data[0..32]);
        let net = U256::from_big_endian(&log.data[64..96]);
        let fee = U256::from_big_endian(&log.data[96..128]);
        let to_address = Address::from_slice(&log.data[160..180]);

        Ok(Self { token_address, sender, to_chain_id, to_address, amount: net, fee })
    }
}

abigen!(Proxy, "./src/types/proxy.json", event_derives(serde::Deserialize, serde::Serialize));
abigen!(Lock, "./src/types/lock.json", event_derives(serde::Deserialize, serde::Serialize));

#[test]
fn test_decode_log() {
    let fee_bytes = Bytes::from_str("0x0000000000000000000000007927ead16ae53d91ca4ef579b8493ddb3eb5d4650000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000002a3078303030303030303030303030303030303030303030303030303030303030303030303030303030300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000602a3078303030303030303030303030303030303030303030303030303030303030303030303030303030301421cf4ff16099d45b24eb1043fb0b2b76ad6fbeda00b8fb0501090000000000000000000000000000000000000000000000000000").unwrap();
    let fee_topics =
        vec![H256::from_str("0xcdb9fb741d82c65a081bb855b5e42174193549c537fd57a199609593827cff71").unwrap()];
    println!("{:?}", fee_bytes.len());
    let fee_log = abi::RawLog { topics: fee_topics, data: fee_bytes.to_vec() };

    let old_payload = Bytes::from_str("0A7A6B4554482E616C656F3F616C656F317977686C34637A6B703264357177747A737135706D6139676C666137766A32747977653575396D747A6834723932377666353873646C38347074000004BFC91B8E00000000000000000000000000000000000000000000000000").unwrap();
    let mut deser = ZeroCopyWriter::from(old_payload.to_vec());
    let to_asset_addr = String::from_utf8(deser.read_next_bytes()).unwrap();
    let to_addr = String::from_utf8(deser.read_next_bytes()).unwrap();
    let _amount = deser.read_u256();

    println!("{:?}", to_asset_addr);
    println!("{:?}", to_addr);
    println!("{:?}", _amount);

    let payload = EventPayload::decode_log(&fee_log).unwrap();
    println!("{:?}", payload);

    // let mut deser = ZeroCopyWriter::from(payload.to_vec());
    // let to_asset_addr = Address::from_slice(&deser.read_next_bytes());
    // let to_addr = Address::from_slice(&deser.read_next_bytes());
    // let _amount = deser.read_u256();

    // println!("{:?}", to_asset_addr);
    // println!("{:?}", to_addr);
    // println!("{:?}", _amount);
}

#[test]
fn test_decode_log2() {
    let log_bytes = Bytes::from_str("0x000000000000000000000000e5babf57e90f9e219a881d24789f742ccab6f6b100000000000000000000000000000000000000000000000000000000000ad575000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000a7a6b4554482e616c656f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006b0a7a6b4554482e616c656f3f616c656f317977686c34637a6b703264357177747a737135706d6139676c666137766a32747977653575396d747a6834723932377666353873646c38347074000004bfc91b8e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let fee_topics =
        vec![H256::from_str("0xcdb9fb741d82c65a081bb855b5e42174193549c537fd57a199609593827cff71").unwrap()];

    let fee_log = abi::RawLog { topics: fee_topics, data: log_bytes.to_vec() };
    let payload = EventPayload::decode_log(&fee_log).unwrap();

    println!("{:?}", payload);
}

#[derive(Debug, Clone)]
pub struct EventPayload {
    pub sender: Address,
    pub nonce: U256,
    pub dst_chain_id: u16,
    pub destination: Bytes,
    pub payload: Bytes,
}

impl EthLogDecode for EventPayload {
    fn decode_log(log: &abi::RawLog) -> std::result::Result<Self, abi::Error>
    where
        Self: Sized,
    {
        let packet = PacketFilter::decode_log(log)?;
        let sender = packet.sender;
        let nonce = packet.nonce;
        let dst_chain_id = packet.dst_chain_id;
        let destination = packet.destination;
        let payload = packet.payload;
        Ok(Self { sender, nonce, dst_chain_id, destination, payload })
    }
}

pub struct EthAddress<N: Network> {
    inner: Address,
    phantom: std::marker::PhantomData<N>,
}

impl<N: Network> From<Address> for EthAddress<N> {
    fn from(addr: Address) -> Self {
        Self { inner: addr, phantom: std::marker::PhantomData }
    }
}

impl<N: Network> ToString for EthAddress<N> {
    fn to_string(&self) -> String {
        format!("{:#020x}", self.inner)
    }
}

impl<N: Network> FromField for EthAddress<N> {
    type Field = Field<N>;

    fn from_field(field: &Self::Field) -> anyhow::Result<Self> {
        let bytes = field.to_bytes_le()?;

        let mut buf = [0u8; 20];
        buf.copy_from_slice(bytes[..20].as_ref());
        Ok(Self { inner: Address::from_slice(&buf), phantom: std::marker::PhantomData })
    }
}

impl<N: Network> ToField for EthAddress<N> {
    type Field = Field<N>;

    fn to_field(&self) -> snarkvm_console::prelude::Result<Self::Field> {
        let mut buf = [0u8; 32];
        buf[0..20].copy_from_slice(self.inner.as_bytes());
        Field::from_bytes_le(&buf)
    }
}

#[test]
fn test_eth_address() {
    use std::str::FromStr;
    let eth_addr = "0x96d1B7Cb9De6c951F94de59d15544391c8fD8883";
    let addr = Address::from_str(eth_addr).unwrap();
    let ethaddr = EthAddress::<aleo_rust::Testnet3>::from(addr);

    let field = ethaddr.to_field().unwrap();
    println!("{:?}", field);
    let new_addr = EthAddress::<aleo_rust::Testnet3>::from_field(&field).unwrap();
    assert_eq!(new_addr.inner, addr);
}
