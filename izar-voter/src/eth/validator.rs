use ethers::{
    abi::{self, Token},
    types::{Address, U256},
    utils,
};
use izar_core::{
    network::eth::EthNetwork,
    types::{cert::Certificate, transaction::IzarTransaction},
};

use crate::Validator;

use super::connector::EthConnector;

impl<E: EthNetwork> Validator for EthConnector<E> {
    fn sign(&self, msg: IzarTransaction) -> anyhow::Result<Certificate> {
        let lock_addr: Address = E::LOCK_CONTRACT.parse()?;
        let payload_data = msg.eth_payload()?;
        let IzarTransaction { from_chain_id, from_addr, to_chain_id, nonce, .. } = msg;
        let nonce = U256::from_dec_str(&nonce)?;
        let mut nonce_be = [0u8; 32];
        nonce.to_big_endian(&mut nonce_be);

        let encoded = abi::encode_packed(&[
            Token::Bytes((from_chain_id as u16).to_be_bytes().to_vec()), // src_chain_id
            Token::Bytes((to_chain_id as u16).to_be_bytes().to_vec()),   // to_chain_id
            Token::Bytes(nonce_be.to_vec()),                             // nonce
            Token::Bytes(from_addr.as_bytes().to_vec()),                 // src_address
            Token::Address(lock_addr),                                   // lock_address
            Token::Bytes(payload_data),                                  // payload
        ])?;

        let hash = utils::keccak256(encoded);
        let signatrue = self.client().signer().sign_hash(hash.into())?.to_string();
        let signer = E::format_str(self.address());

        Ok(Certificate { signature: signatrue, signer })
    }
}
