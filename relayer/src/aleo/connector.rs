use std::{str::FromStr, sync::Arc};

use aleo_rust::{
    Address, AleoAPIClient, Block, Ciphertext, Credits, Network, Plaintext, PrivateKey, ProgramManager, Record, Value,
    ViewKey,
};
use backon::{BlockingRetryable, ExponentialBuilder};
use izar_core::{
    db::{map::DBMap, RocksDB},
    network::aleo::AleoNetworkExt,
    utils::PlaintextCodec,
};
use rayon::prelude::*;
use tracing::error_span;

use crate::DB_PATH;

#[derive(Clone)]
pub struct AleoOperator<N: Network> {
    pm: Arc<ProgramManager<N>>,
    private_key: PrivateKey<N>,
    view_key: ViewKey<N>,
    current_height: DBMap<u16, u32>,
    unspent_records: DBMap<String, Record<N, Plaintext<N>>>,
    client: reqwest::Client,
}

impl<N: Network> AleoOperator<N> {
    pub fn new(dest: Option<String>, pk: PrivateKey<N>, from_height: Option<u32>) -> anyhow::Result<Self> {
        let client = match dest {
            Some(base_url) => AleoAPIClient::new(&base_url, "testnet3")?,
            None => AleoAPIClient::local_testnet3("3030"),
        };

        let view_key = ViewKey::try_from(&pk)?;
        let pm = ProgramManager::new(Some(pk), None, Some(client.clone()), None, true)?;

        let unspent_records = RocksDB::open_map(DB_PATH, "unspent_records")?;
        let current_height = RocksDB::open_map(DB_PATH, "aleo-chains")?;

        let cur = current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        if let Some(from_height) = from_height {
            if cur < from_height {
                current_height.insert(N::ID, from_height)?;
            }
        }
        let client = reqwest::Client::new();
        Ok(Self { pm: Arc::new(pm), private_key: pk, view_key, unspent_records, current_height, client })
    }

    pub fn client(&self) -> anyhow::Result<&AleoAPIClient<N>> {
        self.pm.api_client()
    }

    pub fn pm(&self) -> Arc<ProgramManager<N>> {
        self.pm.clone()
    }

    pub fn sync(&self) -> anyhow::Result<()> {
        let cur = self.current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        let latest = self.client()?.latest_height()?;
        tracing::info!("sync aleo from {} to {}", cur, latest);

        const BATCH_SIZE: usize = 45;

        if cur >= latest {
            return Ok(());
        }

        for start in (cur..latest).step_by(BATCH_SIZE) {
            let end = (start + BATCH_SIZE as u32).min(latest);
            tracing::warn!("Fetched aleo blocks from {} to {}", start, end);
            self.client()?.get_blocks(start, end)?.into_iter().for_each(|b| {
                if let Err(e) = self.handle_credits(&b) {
                    tracing::error!("handle credits error: {:?}", e);
                }
            });
        }

        self.current_height.insert(N::ID, latest)?;

        Ok(())
    }

    // TODO: use tokio sync
    pub fn fast_sync(&self) -> anyhow::Result<()> {
        let cur = self.current_height.get(&N::ID)?.unwrap_or(N::START_HEIGHT);
        let latest = self.client()?.latest_height()?;
        tracing::info!("sync aleo from {} to {}", cur, latest);

        if cur >= latest {
            return Ok(());
        }

        const BATCH_SIZE: usize = 500;

        for start in (cur..latest).step_by(BATCH_SIZE) {
            let end = (start + BATCH_SIZE as u32).min(latest);

            self.fast_get_blocks(start, end)?.into_iter().for_each(|b| {
                if let Err(e) = self.handle_credits(&b) {
                    tracing::error!("handle credits error: {:?}", e);
                }
            });
        }

        self.current_height.insert(N::ID, latest)?;

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
                self.client()?.get_blocks(*st, *ed)
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
            if let Err(e) = self_clone.sync() {
                tracing::error!("failed to sync aleo: {}", e);
            }

            std::thread::sleep(std::time::Duration::from_secs(15));
        });

        self
    }

    pub fn sync_and_initial(self) -> Self {
        error_span!("ALEO").in_scope(|| {
            let op = || self.fast_sync();
            op.retry(&ExponentialBuilder::default()).call().expect("sync aleo failed");
            self.initial()
        })
    }

    pub fn get_record(&self) -> anyhow::Result<(String, Record<N, Plaintext<N>>)> {
        let record = self.unspent_records.iter().next();
        let (rid, record) = record.ok_or(anyhow::anyhow!("no fee record"))?;

        Ok((rid.into_owned(), record.into_owned()))
    }

    fn handle_credits(&self, block: &Block<N>) -> anyhow::Result<()> {
        // handle in
        block.clone().into_serial_numbers().for_each(|sn| {
            let _ = self.unspent_records.remove(&sn.to_string());
        });
        // handle out
        for (commit, record) in block.clone().into_records() {
            if !record.is_owner(&self.view_key) {
                continue;
            }
            let sn = Record::<N, Ciphertext<N>>::serial_number(self.private_key, commit)?;
            let record = record.decrypt(&self.view_key)?;
            if let Ok(credits) = record.microcredits() {
                if credits > N::ALEO_FEE_LIMIT {
                    tracing::info!("got a new record {:?}", record);
                    self.unspent_records.insert(sn.to_string(), record)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_current_keepers(&self) -> anyhow::Result<Vec<String>> {
        let flag = Plaintext::from_str("true")?;
        let keepers_value = self.client()?.get_mapping_value(N::ALEO_PROTOCOL_CONTRACT, "izar_keeper", flag)?;
        if let Value::Plaintext(Plaintext::Array(arr, _)) = keepers_value {
            arr.into_iter()
                .map(|p| {
                    let addr: Address<N> = PlaintextCodec::<N>::decode(&p)?;
                    Ok(addr.to_string())
                })
                .collect::<anyhow::Result<Vec<String>>>()
        } else {
            anyhow::bail!("invalid keepers")
        }
    }

    pub fn reqwest_client(&self) -> reqwest::Client {
        self.client.clone()
    }
}
