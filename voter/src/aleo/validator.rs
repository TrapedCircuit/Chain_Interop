use aleo_rust::Network;
use izar_core::{
    types::{aleo::IzarRecvMsg, cert::Certificate, transaction::IzarTransaction},
    utils::PlaintextCodec,
    ToFields,
};

use crate::Validator;

use super::connector::AleoConnector;

impl<N: Network> Validator for AleoConnector<N> {
    fn sign(&self, msg: IzarTransaction) -> anyhow::Result<Certificate> {
        let payload = IzarRecvMsg::try_from(&msg)?.encode()?;
        let fields = payload.to_fields()?;
        let mut rng = rand::thread_rng();

        let signature = self.private_key().sign(&fields, &mut rng)?.to_string();
        let signer = self.address().to_string();

        Ok(Certificate { signature, signer })
    }
}
