use aleo_rust::{Field, Network};
use snarkvm_utilities::{FromBytes, ToBytes};

pub trait AleoNetworkExt {
    const IZAR_CHAIN_ID: u32 = 2;
    const ALEO_PROTOCOL_CONTRACT: &'static str = "izar_protocol_v1.aleo";
    const ALEO_PROXY_CONTRACT: &'static str = "izar_token_proxy_v1.aleo";
    const ALEO_SPEEDUP_CONTRACT: &'static str = "izar_speedup_v2.aleo";
    const START_HEIGHT: u32 = 1699883;
    const ALEO_FEE_LIMIT: u64 = 1000000;
    const ALEO_PRIORITY_FEE: u64 = 10000;

    fn format_input_array<T: ToString>(arr: &[T]) -> String {
        let mut res = String::new();
        res.push('[');
        for (i, v) in arr.iter().enumerate() {
            res.push_str(&v.to_string());
            if i != arr.len() - 1 {
                res.push(',');
            }
        }
        res.push(']');
        res
    }

    fn bytes_to_field<N: Network>(bytes: &[u8]) -> anyhow::Result<Field<N>> {
        let mut bytes = bytes.to_vec();
        bytes.resize(Field::<N>::SIZE_IN_BYTES, 0);

        Field::from_bytes_le(&bytes)
    }

    fn bytes_from_field<N: Network>(field: &Field<N>) -> anyhow::Result<Vec<u8>> {
        let bytes = field.to_bytes_le()?;
        Ok(bytes[..Field::<N>::SIZE_IN_BYTES].to_vec())
    }
}

impl<N: Network> AleoNetworkExt for N {}
