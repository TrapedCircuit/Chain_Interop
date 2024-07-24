use std::sync::Arc;

use rocksdb::WriteBatch;
use serde::{de::DeserializeOwned, Serialize};

use super::iter::{Iter, Keys, Values};

#[derive(Clone)]
pub struct DBMap<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    pub inner: Arc<rocksdb::DB>,
    pub(crate) prefix: Vec<u8>,
    pub(crate) _marker: std::marker::PhantomData<(K, V)>,
}

impl<'a, K: 'a + Serialize + DeserializeOwned, V: 'a + Serialize + DeserializeOwned> DBMap<K, V> {
    pub fn inner(&self) -> Arc<rocksdb::DB> {
        self.inner.clone()
    }
    pub fn insert(&self, key: K, value: V) -> anyhow::Result<()> {
        let raw_key = self.prefix_key(&key)?;
        let raw_value = bincode::serialize(&value)?;
        self.inner.put(raw_key, raw_value)?;

        Ok(())
    }

    pub fn remove(&self, key: &K) -> anyhow::Result<()> {
        let raw_key = self.prefix_key(key)?;
        self.inner.delete(raw_key)?;

        Ok(())
    }

    pub fn get(&self, key: &K) -> anyhow::Result<Option<V>> {
        let raw_key = self.prefix_key(key)?;

        let value = self.inner.get(raw_key)?;

        if let Some(value) = value {
            let value = bincode::deserialize(&value)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn take(&self, key: &K) -> anyhow::Result<Option<V>> {
        let raw_key = self.prefix_key(key)?;
        let value = self.inner.get(&raw_key)?;

        if let Some(value) = value {
            let value = bincode::deserialize(&value)?;
            self.inner.delete(raw_key)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn contain(&self, key: &K) -> anyhow::Result<bool> {
        let raw_key = self.prefix_key(key)?;
        let value = self.inner.get(raw_key)?;

        Ok(value.is_some())
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(self.prefix.clone(), self.inner.prefix_iterator(self.prefix.clone()))
    }

    pub fn keys(&self) -> Keys<K> {
        Keys::new(self.prefix.clone(), self.inner.prefix_iterator(self.prefix.clone()))
    }

    pub fn values(&self) -> Values<V> {
        Values::new(self.inner.prefix_iterator(self.prefix.clone()))
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<(K, V)>> {
        let mut result = Vec::new();
        let iter = self.inner.prefix_iterator(self.prefix.clone());
        for item in iter {
            let (key, value) = item?;
            if key.starts_with(&self.prefix) {
                let key = &key[self.prefix.len()..];
                let key = bincode::deserialize(key)?;
                let value = bincode::deserialize(&value)?;

                result.push((key, value));
            }
        }

        Ok(result)
    }

    pub fn prefix_key(&self, key: &K) -> anyhow::Result<Vec<u8>> {
        let key_bytes = bincode::serialize(key)?;
        Ok([self.prefix.clone(), key_bytes].concat())
    }

    pub fn write_append(&self, key: K, value: V, batch: &mut WriteBatch) -> anyhow::Result<()> {
        let key_bytes = bincode::serialize(&key)?;
        let value_bytes = bincode::serialize(&value)?;
        batch.put([self.prefix.clone(), key_bytes].concat(), value_bytes.clone());
        Ok(())
    }

    pub fn delete_append(&self, key: &K, batch: &mut WriteBatch) -> anyhow::Result<()> {
        let key_bytes = bincode::serialize(key)?;
        batch.delete([self.prefix.clone(), key_bytes].concat());
        Ok(())
    }
}
