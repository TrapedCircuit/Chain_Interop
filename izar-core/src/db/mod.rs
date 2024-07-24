pub mod iter;
pub mod map;

use std::{path::Path, sync::Arc};

use once_cell::sync::OnceCell;
use rocksdb::WriteBatch;
use serde::{de::DeserializeOwned, Serialize};

use self::map::DBMap;

#[derive(Clone)]
pub struct RocksDB(Arc<rocksdb::DB>);

impl RocksDB {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        static DB: OnceCell<RocksDB> = OnceCell::new();

        // Retrieve the database.
        let database = DB
            .get_or_try_init(|| {
                // Customize database options.
                let mut options = rocksdb::Options::default();
                options.set_compression_type(rocksdb::DBCompressionType::Lz4);

                let rocksdb = {
                    options.increase_parallelism(2);
                    options.set_max_background_jobs(4);
                    options.create_if_missing(true);

                    Arc::new(rocksdb::DB::open(&options, path)?)
                };

                Ok::<_, anyhow::Error>(RocksDB(rocksdb))
            })?
            .clone();

        Ok(database)
    }

    pub fn open_map<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned>(
        path: impl AsRef<Path>,
        prefix: &str,
    ) -> anyhow::Result<DBMap<K, V>> {
        let db = Self::open(path)?;

        let prefix = prefix.as_bytes().to_vec();

        Ok(DBMap { inner: db.inner(), prefix, _marker: std::marker::PhantomData })
    }

    pub fn atomic_batch(
        inner: Arc<rocksdb::DB>,
        closure: impl FnOnce(&mut WriteBatch) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        closure(&mut batch)?;
        inner.write(batch)?;
        Ok(())
    }

    pub fn inner(&self) -> Arc<rocksdb::DB> {
        self.0.clone()
    }
}
