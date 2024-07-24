use std::{str::FromStr, time::Duration};

use aleo_rust::Network;
use clap::Parser;
use izar_core::network::{aleo::AleoNetworkExt, eth::EthNetwork, testnet::IzarTestnet, IzarNetwork};
use izar_relayer::{aleo::connector::AleoOperator, eth::connector::EthOperator, relayer::IzarRelayer};
use serde::Deserialize;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long, default_value = "./relayer.toml")]
    pub config: String,

    #[clap(long, default_value = "false")]
    pub mainnet: bool,
}

#[derive(Debug, Deserialize)]
pub struct RelayerConfig {
    pub api_dest: String,
    pub port: u16,
    pub metrics: String,
    // nodes configs
    pub aleo_config: AleoConfig,
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
    pub fn parse<N: Network>(self) -> AleoOperator<N> {
        let pk: aleo_rust::PrivateKey<N> = aleo_rust::PrivateKey::<N>::from_str(&self.pk).expect("parse pk");
        AleoOperator::<N>::new(self.dest, pk, self.from_height).expect("init aleo")
    }
}

#[derive(Debug, Deserialize)]
pub struct EthConfig {
    pk: String,
    dest: String,
}

impl EthConfig {
    pub fn parse<E: EthNetwork>(self) -> EthOperator<E> {
        EthOperator::<E>::new(self.pk, self.dest).expect("eth init")
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // init logger
    tracing_subscriber::fmt().init();
    let config_str = std::fs::read_to_string(cli.config).expect("read config file");
    let config: RelayerConfig = toml::from_str(&config_str).expect("parse config");
    tracing::info!("relayer init with {:#?}", config);
    // init metrics
    if let Err(e) = izar_core::metrics::metrics_init(&config.metrics, Duration::from_secs(30)) {
        tracing::error!("metrics init error: {}", e);
    }

    if cli.mainnet {
        panic!("not support mainnet")
    } else {
        init::<IzarTestnet>(config).await;
    }
}

async fn init<I: IzarNetwork>(config: RelayerConfig) {
    // init izar operator
    let port = config.port;
    let api_dest = format!("{}/api/v1/BridgeTx", config.api_dest);
    let mut operators = IzarRelayer::<I>::new(port, api_dest);

    // init aleo operator
    let aleo_op = config.aleo_config.parse::<I::Aleo>();
    operators.insert_operator(I::Aleo::IZAR_CHAIN_ID, Box::new(aleo_op));

    // init sepolia operator
    if let Some(config) = config.sepolia_config {
        operators.insert_operator(I::Sepolia::IZAR_CHAIN_ID, Box::new(config.parse::<I::Sepolia>()));
    }

    // init linea operator
    if let Some(config) = config.linea_config {
        operators.insert_operator(I::Linea::IZAR_CHAIN_ID, Box::new(config.parse::<I::Linea>()));
    }

    // init goerli operator
    if let Some(config) = config.goerli_config {
        operators.insert_operator(I::Goerli::IZAR_CHAIN_ID, Box::new(config.parse::<I::Goerli>()));
    }

    // init zksync operator
    if let Some(config) = config.zksync_config {
        operators.insert_operator(I::Zksync::IZAR_CHAIN_ID, Box::new(config.parse::<I::Zksync>()));
    }

    // init scroll operator
    if let Some(config) = config.scroll_config {
        operators.insert_operator(I::Scroll::IZAR_CHAIN_ID, Box::new(config.parse::<I::Scroll>()));
    }

    // init optimism operator
    if let Some(config) = config.optimism_config {
        operators.insert_operator(I::Optimism::IZAR_CHAIN_ID, Box::new(config.parse::<I::Optimism>()));
    }

    // init arbitrum operator
    if let Some(config) = config.arbitrum_config {
        operators.insert_operator(I::Arbitrum::IZAR_CHAIN_ID, Box::new(config.parse::<I::Arbitrum>()));
    }

    // init taiko operator
    if let Some(config) = config.taiko_config {
        operators.insert_operator(I::Taiko::IZAR_CHAIN_ID, Box::new(config.parse::<I::Taiko>()));
    }

    // init operators
    operators.initial().await.expect("initial");
}
