pub mod execute;
pub mod rest;
pub mod store;
pub mod types;

use izar_core::network::IzarNetwork;
use std::{collections::HashMap, sync::Arc};

use crate::{Operator, Operators};

use self::store::RelayerStore;

pub struct IzarRelayer<I: IzarNetwork> {
    port: u16,
    sequencer: String,
    store: RelayerStore,
    operators: Operators<I>,
    client: reqwest::Client,
}

impl<I: IzarNetwork> IzarRelayer<I> {
    pub fn new(port: u16, sequencer: String) -> Self {
        Self {
            port,
            sequencer,
            store: RelayerStore::build().expect("Failed to open relayer store"),
            operators: HashMap::new(),
            client: reqwest::Client::new(),
        }
    }

    pub fn insert_operator(&mut self, chain_id: u32, operator: Box<dyn Operator<I>>) {
        self.operators.insert(chain_id, operator);
    }

    pub fn store(&self) -> &RelayerStore {
        &self.store
    }

    pub fn sequencer(&self) -> &str {
        &self.sequencer
    }

    pub fn client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn initial(self) -> anyhow::Result<()> {
        let izar = Arc::new(self);

        // start execute handler
        let executor = izar.clone();
        tokio::spawn(async move {
            if let Err(e) = IzarRelayer::execute_handler(executor).await {
                tracing::error!("execute handler exit: {}", e);
            }
        });

        // start pending checker
        let checker = izar.clone();
        tokio::spawn(async move { IzarRelayer::pending_checker(checker) });

        // start rest server
        IzarRelayer::serve(izar).await
    }
}
