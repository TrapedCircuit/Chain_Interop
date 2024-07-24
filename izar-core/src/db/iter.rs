use std::{borrow::Cow, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};

pub struct Iter<'a, K: 'a + Serialize + DeserializeOwned, V: 'a + Serialize + DeserializeOwned> {
    prefix: Vec<u8>,
    db_iter: rocksdb::DBIterator<'a>,
    _phantom: PhantomData<(K, V)>,
}

impl<'a, K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Iter<'a, K, V> {
    pub fn new(prefix: Vec<u8>, db_iter: rocksdb::DBIterator<'a>) -> Self {
        Self { prefix, db_iter, _phantom: PhantomData }
    }
}

impl<'a, K: Serialize + DeserializeOwned + Clone, V: Serialize + DeserializeOwned + Clone> Iterator for Iter<'a, K, V> {
    type Item = (Cow<'a, K>, Cow<'a, V>);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self
            .db_iter
            .next()?
            .map_err(|e| {
                tracing::error!("RocksDB Iter iterator error: {e}");
            })
            .ok()?;

        // Deserialize the key and value.
        let key = bincode::deserialize(&key[self.prefix.len()..])
            .map_err(|e| {
                tracing::error!("RocksDB Iter deserialize(key) error: {e}");
            })
            .ok()?;

        let value = bincode::deserialize(&value)
            .map_err(|e| {
                tracing::error!("RocksDB Iter deserialize(value) error: {e}");
            })
            .ok()?;

        Some((Cow::Owned(key), Cow::Owned(value)))
    }
}

/// An iterator over the keys of a prefix.
pub struct Keys<'a, K: 'a + Serialize + DeserializeOwned> {
    prefix: Vec<u8>,
    db_iter: rocksdb::DBIterator<'a>,
    _phantom: PhantomData<K>,
}

impl<'a, K: 'a + Serialize + DeserializeOwned> Keys<'a, K> {
    pub(crate) fn new(prefix: Vec<u8>, db_iter: rocksdb::DBIterator<'a>) -> Self {
        Self { prefix, db_iter, _phantom: PhantomData }
    }
}

impl<'a, K: 'a + Clone + Serialize + DeserializeOwned> Iterator for Keys<'a, K> {
    type Item = Cow<'a, K>;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, _) = self
            .db_iter
            .next()?
            .map_err(|e| {
                tracing::error!("RocksDB Keys iterator error: {e}");
            })
            .ok()?;

        // Deserialize the key.
        let key = bincode::deserialize(&key[self.prefix.len()..])
            .map_err(|e| {
                tracing::error!("RocksDB Keys deserialize(key) error: {e}");
            })
            .ok()?;

        Some(Cow::Owned(key))
    }
}

/// An iterator over the values of a prefix.
pub struct Values<'a, V: 'a + Serialize + DeserializeOwned> {
    db_iter: rocksdb::DBIterator<'a>,
    _phantom: PhantomData<V>,
}

impl<'a, V: 'a + Serialize + DeserializeOwned> Values<'a, V> {
    pub(crate) fn new(db_iter: rocksdb::DBIterator<'a>) -> Self {
        Self { db_iter, _phantom: PhantomData }
    }
}

impl<'a, V: 'a + Clone + Serialize + DeserializeOwned> Iterator for Values<'a, V> {
    type Item = Cow<'a, V>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = self
            .db_iter
            .next()?
            .map_err(|e| {
                tracing::error!("RocksDB Values iterator error: {e}");
            })
            .ok()?;

        // Deserialize the value.
        let value = bincode::deserialize(&value)
            .map_err(|e| {
                tracing::error!("RocksDB Values deserialize(value) error: {e}");
            })
            .ok()?;

        Some(Cow::Owned(value))
    }
}
