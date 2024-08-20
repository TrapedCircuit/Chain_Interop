use aleo_rust::Network;
use async_trait::async_trait;
use izar_core::{
    network::{aleo::AleoNetworkExt, IzarNetwork},
    types::{aleo::IzarRecvMsg, transaction::IzarTransaction},
    utils::PlaintextCodec,
};

use crate::Operator;

use super::{connector::AleoOperator, INVALID_SIGN, INVALID_VALIDATOR};

#[async_trait]
impl<I: IzarNetwork, N: Network> Operator<I> for AleoOperator<N> {
    async fn execute(&self, tx: &IzarTransaction) -> anyhow::Result<String> {
        let payload = IzarRecvMsg::<N>::try_from(tx)?.encode()?;
        let keepers = self.get_current_keepers()?;
        let mut signatures = vec![INVALID_SIGN.to_string(); keepers.len()];
        for c in tx.certificates.iter() {
            for (i, k) in keepers.iter().enumerate() {
                if k == INVALID_VALIDATOR {
                    continue;
                }
                if k == &c.signer {
                    signatures[i].clone_from(&c.signature);
                }
            }
        }
        let inputs = [N::format_input_array(&signatures), N::format_input_array(&keepers), payload.to_string()];

        let result = self.pm().execute_program(
            N::ALEO_PROXY_CONTRACT,
            "receive_payload",
            inputs.iter(),
            N::ALEO_PRIORITY_FEE,
            None,
            None,
            None,
        )?;

        Ok(result)
    }
}
