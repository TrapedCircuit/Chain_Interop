use std::{str::FromStr, time::Duration};

use aleo_rust::{Address, AleoAPIClient, Block, Identifier, Network, PrivateKey, ProgramID, ViewKey};
use backon::{BlockingRetryable, ExponentialBuilder};
use izar_core::{
    db::{map::DBMap, RocksDB},
    network::aleo::AleoNetworkExt,
    types::{
        aleo::{AleoTransaction, EthRawHash, IzarCrossMsg},
        transaction::IzarTransaction,
    },
    utils::PlaintextCodec,
    Argument, Input, Output, Transition,
};
use rayon::prelude::*;
use tracing::error_span;

use crate::{DB_PATH, SPEEDUP_TXS, UNCONFIRMED_TXS};

use super::filter::TransitionFilter;

#[derive(Clone)]
pub struct AleoConnector<N: Network> {
    private_key: PrivateKey<N>,
    view_key: ViewKey<N>,
    address: Address<N>,
    aleo_client: AleoAPIClient<N>,
    unconfirmed_txs: DBMap<String, IzarTransaction>,
    speedup_txs: DBMap<String, String>,
    current_height: DBMap<u16, u32>,
    filter: TransitionFilter<N>,
}

impl<N: Network> AleoConnector<N> {
    pub fn new(pk: PrivateKey<N>, dest: Option<String>, from_height: Option<u32>) -> anyhow::Result<Self> {
        let aleo_client = match dest {
            Some(dest) => AleoAPIClient::new(&dest, "testnet3")?,
            None => AleoAPIClient::testnet3(),
        };

        let vk = ViewKey::try_from(pk)?;
        let address = Address::try_from(pk)?;

        let unconfirmed_txs = RocksDB::open_map(DB_PATH, UNCONFIRMED_TXS)?;
        let speedup_txs = RocksDB::open_map(DB_PATH, SPEEDUP_TXS)?;
        let current_height = RocksDB::open_map(DB_PATH, "aleo-chains")?;
        let cur = current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        if let Some(from_height) = from_height {
            if cur < from_height {
                current_height.insert(N::ID, from_height)?;
            }
        }

        let proxy_program = ProgramID::<N>::from_str(N::ALEO_PROXY_CONTRACT)?;
        let protocol_program = ProgramID::<N>::from_str(N::ALEO_PROTOCOL_CONTRACT)?;
        let speedup_program = ProgramID::<N>::from_str(N::ALEO_SPEEDUP_CONTRACT)?;
        let filter = TransitionFilter::default().add_programs(vec![proxy_program, protocol_program, speedup_program]);

        Ok(Self {
            address,
            private_key: pk,
            view_key: vk,
            aleo_client,
            unconfirmed_txs,
            speedup_txs,
            current_height,
            filter,
        })
    }

    pub fn aleo_client(&self) -> &AleoAPIClient<N> {
        &self.aleo_client
    }

    pub fn sync(&self) -> anyhow::Result<()> {
        let cur = self.current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        let latest = self.aleo_client().latest_height()?;

        const BATCH_SIZE: u32 = 50;

        if cur >= latest {
            return Ok(());
        }

        for start in (cur..latest).step_by(BATCH_SIZE as usize) {
            let end = (start + BATCH_SIZE).min(latest);
            tracing::warn!("fetching aleo blocks from {} to {}", start, end);
            self.aleo_client()
                .get_blocks(start, end)?
                .into_iter()
                .flat_map(|b| self.filter.filter_block_with_txid(b))
                .for_each(|(tid, t)| self.transition_handler(tid, t));
            self.current_height.insert(N::ID, end)?;
        }

        Ok(())
    }

    // TODO: use futures
    pub fn fast_sync(&self) -> anyhow::Result<()> {
        let cur = self.current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        let latest = self.aleo_client.latest_height()?;
        tracing::info!("sync aleo from {} to {}", cur, latest);

        if cur >= latest {
            return Ok(());
        }

        const BATCH_SIZE: usize = 500;

        for start in (cur..latest).step_by(BATCH_SIZE) {
            let end = (start + BATCH_SIZE as u32).min(latest);

            self.fast_get_blocks(start, end)?.into_iter().for_each(|b| {
                let transactions = self.filter.filter_block_with_txid(b);
                for (tid, t) in transactions {
                    self.transition_handler(tid, t);
                }
            });

            self.current_height.insert(N::ID, end)?;
        }

        Ok(())
    }

    fn fast_get_blocks(&self, st: u32, ed: u32) -> anyhow::Result<Vec<Block<N>>> {
        const BATCH_SIZE: usize = 500;

        if ed - st > BATCH_SIZE as u32 {
            return Err(anyhow::anyhow!("too large batch size"));
        }

        let par_getter = (st..ed).step_by(50).map(|st| (st, (st + 50).min(ed))).collect::<Vec<(u32, u32)>>();

        let blocks = par_getter
            .par_iter()
            .map(|(st, ed)| {
                tracing::warn!("fetching aleo blocks from {} to {}", st, ed);
                self.aleo_client.get_blocks(*st, *ed)
            })
            .collect::<anyhow::Result<Vec<Vec<Block<N>>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<Block<N>>>();

        Ok(blocks)
    }

    fn initial(self) -> Self {
        let self_clone = self.clone();

        std::thread::spawn(move || loop {
            if let Err(e) = self.sync() {
                tracing::error!("Failed to aleo sync: {:?}", e);
            }
            std::thread::sleep(Duration::from_secs(10));
        });

        self_clone
    }

    pub fn sync_and_initial(self) -> Self {
        error_span!("ALEO").in_scope(|| {
            let sync_op = || self.fast_sync();
            if let Err(e) = sync_op.retry(&ExponentialBuilder::default().with_max_times(5)).call() {
                tracing::error!("Failed to aleo fast sync: {:?}", e);
            }
            self.initial()
        })
    }

    pub fn transition_handler(&self, tid: N::TransactionID, t: Transition<N>) {
        let result = match t.function_name().to_string().as_str() {
            "cross_public" => self.handle_cross_public(tid, t),
            "speed_up_eth" => self.handle_speed_up_eth(tid, t),
            _ => Ok(()),
        };
        if let Err(e) = result {
            tracing::error!("Failed to handle transition: {:?}", e);
        }
    }

    pub fn handle_speed_up_eth(&self, tid: N::TransactionID, t: Transition<N>) -> anyhow::Result<()> {
        tracing::info!("got a speedup tx {tid}");
        let inputs = t.inputs();
        if let Input::Public(_, Some(p)) = &inputs[0] {
            let tx_hash: EthRawHash = PlaintextCodec::<N>::decode(p)?;
            let tx_hash = format!("{:#032x}", tx_hash.to_h256());
            self.speedup_txs.insert(tid.to_string(), tx_hash)?;
        }

        Ok(())
    }

    pub fn handle_cross_public(&self, tid: N::TransactionID, t: Transition<N>) -> anyhow::Result<()> {
        tracing::info!("got a cross public tx {tid}");
        let input = &t.inputs()[0];
        if let Input::Public(_, Some(p)) = input {
            let msg = IzarCrossMsg::<N>::decode(p)?;
            let from_addr = get_address_from_burn(&t.outputs()[0])?;
            let aleo_tx = AleoTransaction::<N>::new(
                tid,
                msg.to_chain_id,
                msg.to_asset_addr,
                msg.to_addr,
                from_addr,
                msg.amount,
                msg.fee,
            );

            self.unconfirmed_txs.insert(tid.to_string(), aleo_tx.try_into()?)?;
        }
        Ok(())
    }

    pub fn address(&self) -> &Address<N> {
        &self.address
    }

    pub fn private_key(&self) -> &PrivateKey<N> {
        &self.private_key
    }

    pub fn view_key(&self) -> &ViewKey<N> {
        &self.view_key
    }
}

pub fn get_address_from_burn<N: Network>(output: &Output<N>) -> anyhow::Result<Address<N>> {
    if let Output::Future(_, Some(f)) = output {
        let arg = &f.arguments()[1];
        if let Argument::Future(burn) = arg {
            if let Argument::Plaintext(aleo_rust::Plaintext::Struct(s, _)) = &burn.arguments()[0] {
                let holder = s
                    .get(&Identifier::from_str("holder")?)
                    .ok_or_else(|| anyhow::anyhow!("invalid burn transition output"))?;
                PlaintextCodec::<N>::decode(holder)
            } else {
                anyhow::bail!("invalid burn transition output")
            }
        } else {
            anyhow::bail!("invalid burn transition output")
        }
    } else {
        anyhow::bail!("invalid burn transition output")
    }
}
