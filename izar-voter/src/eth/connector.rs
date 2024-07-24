use std::time::Duration;

use anyhow::anyhow;
use ethers::{
    prelude::{parse_log, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Filter},
};

use crate::{DB_PATH, UNCONFIRMED_TXS};
use izar_core::{
    db::{map::DBMap, RocksDB},
    network::eth::EthNetwork,
    types::{
        eth::{EthTransaction, EventFee, EventPayload},
        transaction::IzarTransaction,
    },
    Itertools,
};
use tracing::{error_span, Instrument};

#[derive(Clone)]
pub struct EthConnector<E: EthNetwork> {
    client: SignerMiddleware<Provider<Http>, LocalWallet>,
    unconfrimed_txs: DBMap<String, IzarTransaction>,
    current_height: DBMap<u32, u64>,
    address: Address,

    phantom: std::marker::PhantomData<E>,
}

impl<E: EthNetwork> EthConnector<E> {
    pub fn new(pk: String, dest: String, from_height: Option<u64>) -> anyhow::Result<Self> {
        let wallet = pk.parse::<LocalWallet>()?;
        let provider = Provider::<Http>::try_from(dest)?;
        let address = wallet.address();

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(E::ETH_CHAIN_ID));

        let unconfrimed_txs = RocksDB::open_map(DB_PATH, UNCONFIRMED_TXS)?;
        let current_height = RocksDB::open_map(DB_PATH, "eth-chains")?;

        let cur = current_height.get(&E::ETH_CHAIN_ID)?.unwrap_or(E::START_HEIGHT);
        if let Some(from_height) = from_height {
            if cur < from_height {
                current_height.insert(E::ETH_CHAIN_ID, from_height)?;
            }
        }

        Ok(Self { client, unconfrimed_txs, current_height, address, phantom: std::marker::PhantomData })
    }

    pub async fn sync(&self) -> anyhow::Result<()> {
        let cur_height = self.current_height.get(&E::ETH_CHAIN_ID)?.unwrap_or(E::START_HEIGHT);
        let latest_height = self.client.get_block_number().await?.as_u64();

        if cur_height >= latest_height {
            return Ok(());
        }

        tracing::info!("syncing eth blocks from {} to {}", cur_height, latest_height);
        for cur in (cur_height..latest_height).step_by(E::REQUEST_BLOCK_NUM) {
            let end = (cur + E::REQUEST_BLOCK_NUM as u64).min(latest_height);

            let filter = Filter::new()
                .address(vec![E::PROXY_CONTRACT.parse::<Address>()?, E::WRAPPER_CONTRACT.parse::<Address>()?])
                .events(vec![
                    "WrapperLock(address,address,uint64,bytes,uint256,uint256)",
                    "Packet(address,uint256,uint16,bytes,bytes)",
                ])
                .from_block(cur)
                .to_block(end);

            let logs = self.client.get_logs(&filter).await?;

            for mut chunk in logs.into_iter().chunks(2).into_iter() {
                let payload_log = chunk.next();
                let fee_log = chunk.next();

                match (fee_log, payload_log) {
                    (Some(fee_log), Some(payload_log)) => {
                        let tid = fee_log.transaction_hash.ok_or(anyhow!("no tx hash"))?;
                        let payload_log = parse_log::<EventPayload>(payload_log)?;
                        let fee_log = parse_log::<EventFee>(fee_log)?;

                        let tx = EthTransaction::<E>::from_logs(fee_log, payload_log, tid)?;
                        tracing::info!("got a eth tx {:?}", tx); // TODO
                        self.unconfrimed_txs.insert(E::format_str(tid), tx.try_into()?)?;
                    }
                    _ => tracing::error!("invalid log pair"),
                }
            }
            tracing::warn!("fetched eth blocks from {} to {}", cur, end);
        }

        self.current_height.insert(E::ETH_CHAIN_ID, latest_height)?;

        Ok(())
    }

    pub fn initial(self) -> Self {
        let self_clone = self.clone();
        let fut = async move {
            loop {
                if let Err(e) = self_clone.sync().instrument(error_span!("ETH", network = E::IZAR_CHAIN_ID)).await {
                    tracing::error!("eth sync error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(20)).await;
            }
        };

        tokio::spawn(fut);
        self
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn client(&self) -> &SignerMiddleware<Provider<Http>, LocalWallet> {
        &self.client
    }
}
