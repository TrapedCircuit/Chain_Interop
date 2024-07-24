use aleo_rust::{Address, Field, Literal, Network, Plaintext};
use std::{ops::Deref, str::FromStr};

pub trait PlaintextCodec<N: Network> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized;
}

impl<N: Network> PlaintextCodec<N> for u32 {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Literal(Literal::U32(v), _) = p {
            Ok(*v.deref())
        } else {
            anyhow::bail!("invalid u32 plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{}u32", self);
        Plaintext::from_str(&raw_str)
    }
}

impl<N: Network> PlaintextCodec<N> for u64 {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Literal(Literal::U64(v), _) = p {
            Ok(*v.deref())
        } else {
            anyhow::bail!("invalid u64 plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{}u64", self);
        Plaintext::from_str(&raw_str)
    }
}

impl<N: Network> PlaintextCodec<N> for u128 {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Literal(Literal::U128(v), _) = p {
            Ok(*v.deref())
        } else {
            anyhow::bail!("invalid u128 plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{}u128", self);
        Plaintext::from_str(&raw_str)
    }
}

impl<N: Network> PlaintextCodec<N> for Address<N> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Literal(Literal::Address(v), _) = p {
            Ok(*v)
        } else {
            anyhow::bail!("invalid address plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{}", self);
        Plaintext::from_str(&raw_str)
    }
}

impl<N: Network> PlaintextCodec<N> for Field<N> {
    fn decode(p: &Plaintext<N>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Plaintext::Literal(Literal::Field(v), _) = p {
            Ok(*v)
        } else {
            anyhow::bail!("invalid field plaintext")
        }
    }

    fn encode(&self) -> anyhow::Result<Plaintext<N>>
    where
        Self: Sized,
    {
        let raw_str = format!("{}field", self);
        Plaintext::from_str(&raw_str)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use aleo_rust::{
        Ciphertext, Field, Identifier, Network, Plaintext, PrivateKey, ProgramID, Record, Testnet3, ViewKey,
    };
    use serde_json::json;
    use snarkvm_console::{program::ToFields, types::Group};
    use snarkvm_utilities::{to_bits_le, ToBits};

    #[test]
    fn test_p_struct_from_str() {
        let raw = "{ to_chain_id: 0u32, to_addr: 0field, to_asset_addr: 0field, amount: 100u128, fee: 10u128 }";
        let p = Plaintext::<Testnet3>::from_str(raw).unwrap();
        println!("{:#?}", p);
    }

    #[test]
    fn test_value_vec() {
        let bytes = vec![1u8, 2u8, 3u8];
        let json = json!(
            {
                "bytes": bytes
            }
        );
        let json_str = serde_json::to_string(&json).unwrap();
        println!("{}", json_str);
    }

    #[test]
    fn test_generate_sig() {
        let pk =
            PrivateKey::<Testnet3>::from_str("APrivateKey1zkpDhgXru54MAp2mYgdWqPtfKMdP7ramUh8FVurabjr6ai8").unwrap();
        let payload = Plaintext::<Testnet3>::from_str("{ from_chain_id: 0u32, nonce: 100u128, to_addr: aleo1juuen83htdej22a850l72vewglcscqs32l7f7qyykmq9ywf8muysre5y2m, to_asset_addr: aleo1juuen83htdej22a850l72vewglcscqs32l7f7qyykmq9ywf8muysre5y2m, amount: 10u128 }").unwrap();

        let sig = pk.sign(&payload.to_fields().unwrap(), &mut rand::thread_rng()).unwrap();
        println!("{}", sig);
    }

    #[test]
    fn test_generate_record() {
        let record = Record::<Testnet3, Plaintext<Testnet3>>::from_str(
            "{ owner: aleo1jkhqhs9hnujaf0x4z66f2j6vytgmn7768rvqvkapjn5jvguvvyfq33ytp3.private, microcredits: 100u64.private, _nonce: 2845723700945552257439902855147005550602958568916589593762137374624015164729group.public }",
        ).unwrap();

        let s = Testnet3::hash_to_scalar_psd8(&record.to_fields().unwrap()).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_ethaddr_to_field() {
        let eth_addr = "0x0000000000000000000000000000000000000000".to_string();
        let from_asset_addr = Testnet3::hash_bhp256(&to_bits_le!(eth_addr)).unwrap();
        println!("{}", from_asset_addr);
    }

    #[test]
    fn test_encrypt_decrypt_record() {
        let record = Record::<Testnet3, Plaintext<Testnet3>>::from_str(
            "{ owner: aleo18xhq90m2ezrwh6qut955aw3k8wvgerwdh5alrxsnmt6g88cefqyq7nqs5t.private, microcredits: 100u64.private, _nonce: 2845723700945552257439902855147005550602958568916589593762137374624015164729group.public }",
        ).unwrap();
        let nonce = Group::<Testnet3>::from_str(
            "2845723700945552257439902855147005550602958568916589593762137374624015164729group",
        )
        .unwrap();

        let view_key = ViewKey::<Testnet3>::from_str("AViewKey1djPD41d3EAAZFJqdeUmRnShhJojntxURyJ937XnDwz1n").unwrap();

        let record_view_key = (nonce * *view_key).to_x_coordinate();

        let ciper_record = record.encrypt_symmetric_unchecked(&record_view_key).unwrap();

        let record2 = ciper_record.decrypt(&view_key).unwrap();

        assert_eq!(record, record2);
    }

    #[test]
    fn test_gen_tag() {
        let preimage = Plaintext::<Testnet3>::from_str("[1field, 2field]").unwrap();

        let tag = Testnet3::hash_psd2(&preimage.to_fields().unwrap()).unwrap();
        println!("{}", tag);
    }

    #[test]
    fn test_gen_program_id_scalar() {
        let program_id = ProgramID::<Testnet3>::from_str("izar_token_proxy_v1.aleo").unwrap();
        println!("{}", program_id.to_address().unwrap());

        let scalar = Testnet3::hash_bhp1024(&to_bits_le!(program_id)).unwrap();

        println!("{}", scalar);
    }

    #[test]
    fn test_gen_record_type_scalar() {
        let record_name = Identifier::<Testnet3>::from_str("credits").unwrap();

        let scalar = Testnet3::hash_bhp1024(&to_bits_le!(record_name)).unwrap();

        println!("{}", scalar);
    }

    #[test]
    fn test_cipertext() {
        let view_key = ViewKey::from_str("AViewKey1mbHpTcbVKQhv42GhPsAzijcNPHgmMqiTAUxTwonHU7BG").unwrap();
        let nonce = Group::<Testnet3>::from_str(
            "143424197629526663667703347442709911653829283846283923516849351897515814050group",
        )
        .unwrap();
        let _real_nonce = Field::<Testnet3>::from_str(
            "4884920723389187345066497873846409659255579595219322321646471377087429710673field",
        )
        .unwrap();
        let ciper_address = Ciphertext::<Testnet3>::from_str("ciphertext1qgq07cydg2gu49g556k9zr4a54jcvrws5hzx6zwgzqgraq5lhnwfupexk973ksvgjke4lrxr3wncjfsqprfzjsrsav9eq8cjz09pj9l7qg5cj86f").unwrap();

        let plaintext = ciper_address.decrypt(view_key, nonce).unwrap();
        println!("{}", plaintext);
    }
}
