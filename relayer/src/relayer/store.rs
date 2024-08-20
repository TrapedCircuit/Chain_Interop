use izar_core::{
    db::{map::DBMap, RocksDB},
    types::transaction::IzarTransaction,
};

use crate::DB_PATH;

const EXECUTE_PREFIX: &str = "execute";
const FINALIZE_PREFIX: &str = "finalize";

#[derive(Clone)]
pub struct RelayerStore {
    execute: DBMap<Vec<u8>, IzarTransaction>,
    finalize: DBMap<String, IzarTransaction>,
}

impl RelayerStore {
    pub fn build() -> anyhow::Result<Self> {
        let execute = RocksDB::open_map(DB_PATH, EXECUTE_PREFIX)?;
        let finalize = RocksDB::open_map(DB_PATH, FINALIZE_PREFIX)?;

        Ok(Self { execute, finalize })
    }

    pub fn execute(&self) -> &DBMap<Vec<u8>, IzarTransaction> {
        &self.execute
    }

    pub fn finalize(&self) -> &DBMap<String, IzarTransaction> {
        &self.finalize
    }
}
