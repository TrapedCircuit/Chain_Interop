use std::str::FromStr;

use aleo_rust::Network;
use clap::Parser;
use izar_core::network::{aleo::AleoNetworkExt, eth::EthNetwork, testnet::IzarTestnet, IzarNetwork};
use izar_voter::{aleo::connector::AleoConnector, eth::connector::EthConnector, validators::IzarValidators};
use serde::Deserialize;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long, default_value = "./voter.toml")]
    pub config: String,

    #[clap(long, default_value = "false")]
    pub mainnet: bool,
}

#[derive(Debug, Deserialize)]
pub struct VoterConfig {
    pub api_dest: String,
    pub aleo_config: Option<AleoConfig>,
    pub sepolia_config: Option<EthConfig>,
    pub linea_config: Option<EthConfig>,
    pub zksync_config: Option<EthConfig>,
    pub goerli_config: Option<EthConfig>,
    pub scroll_config: Option<EthConfig>,
    pub optimism_config: Option<EthConfig>,
    pub arbitrum_config: Option<EthConfig>,
    pub taiko_config: Option<EthConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AleoConfig {
    pk: String,
    dest: Option<String>,
    from_height: Option<u32>,
}

impl AleoConfig {
    pub fn parse<N: Network>(self) -> AleoConnector<N> {
        let pk = aleo_rust::PrivateKey::<N>::from_str(&self.pk).expect("parse pk");
        AleoConnector::new(pk, self.dest, self.from_height).expect("init aleo").sync_and_initial()
    }
}

#[derive(Debug, Deserialize)]
pub struct EthConfig {
    pk: String,
    dest: String,
    from_height: Option<u64>,
}

impl EthConfig {
    pub fn parse<E: EthNetwork>(self) -> EthConnector<E> {
        EthConnector::<E>::new(self.pk, self.dest, self.from_height).expect("eth init").initial()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();
    if cli.mainnet {
        panic!("not support mainnet")
    } else {
        init::<IzarTestnet>(cli).await;
    }
}

async fn init<I: IzarNetwork>(cli: Cli) {
    let config_str = std::fs::read_to_string(cli.config).expect("read config file");
    let config: VoterConfig = toml::from_str(&config_str).expect("parse config");
    tracing::info!("voter init with {:#?}", config);
    let mut validators = IzarValidators::new(config.api_dest);

    // init aleo connector
    if let Some(aleo_config) = config.aleo_config {
        tracing::info!("init aleo connector");
        let aleo_conn = Box::new(aleo_config.parse::<I::Aleo>());
        validators.insert_connector(I::Aleo::IZAR_CHAIN_ID, aleo_conn);
    }

    // init sepolia connector
    if let Some(config) = config.sepolia_config {
        tracing::info!("init sepolia connector");
        let conn = Box::new(config.parse::<I::Sepolia>());
        validators.insert_connector(I::Sepolia::IZAR_CHAIN_ID, conn);
    }

    // init linea connector
    if let Some(config) = config.linea_config {
        let conn = Box::new(config.parse::<I::Linea>());
        validators.insert_connector(I::Linea::IZAR_CHAIN_ID, conn);
    }

    // init zksync connector
    if let Some(config) = config.zksync_config {
        let conn = Box::new(config.parse::<I::Zksync>());
        validators.insert_connector(I::Zksync::IZAR_CHAIN_ID, conn);
    }

    // init goerli connector
    if let Some(config) = config.goerli_config {
        let conn = Box::new(config.parse::<I::Goerli>());
        validators.insert_connector(I::Goerli::IZAR_CHAIN_ID, conn);
    }

    // init scroll connector
    if let Some(config) = config.scroll_config {
        let conn = Box::new(config.parse::<I::Scroll>());
        validators.insert_connector(I::Scroll::IZAR_CHAIN_ID, conn);
    }

    // init optimism connector
    if let Some(config) = config.optimism_config {
        let conn = Box::new(config.parse::<I::Optimism>());
        validators.insert_connector(I::Optimism::IZAR_CHAIN_ID, conn);
    }

    // init arbitrum
    if let Some(config) = config.arbitrum_config {
        let conn = Box::new(config.parse::<I::Arbitrum>());
        validators.insert_connector(I::Arbitrum::IZAR_CHAIN_ID, conn);
    }

    // init taiko
    if let Some(taiko_config) = config.taiko_config {
        let conn = Box::new(taiko_config.parse::<I::Taiko>());
        validators.insert_connector(I::Taiko::IZAR_CHAIN_ID, conn);
    }

    // init voters
    validators.initial().await;
}
