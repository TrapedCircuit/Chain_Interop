use std::sync::Arc;

use ethers::{
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};
use izar_core::network::eth::EthNetwork;

#[derive(Clone)]
pub struct EthOperator<E: EthNetwork> {
    pub client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    private_key: String,
    phantom: std::marker::PhantomData<E>,
}

impl<E: EthNetwork> EthOperator<E> {
    pub fn new(pk: String, dest: String) -> anyhow::Result<Self> {
        let wallet = pk.parse::<LocalWallet>()?;
        let provider = Provider::<Http>::try_from(dest)?;

        let client = Arc::new(SignerMiddleware::new(provider, wallet.with_chain_id(E::ETH_CHAIN_ID)));
        Ok(Self { client, private_key: pk, phantom: std::marker::PhantomData })
    }

    pub fn pk(&self) -> &String {
        &self.private_key
    }
}
