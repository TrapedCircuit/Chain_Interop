use aleo_rust::Network;

use self::eth::EthNetwork;
pub mod aleo;
pub mod eth;
pub mod testnet;

pub trait IzarNetwork: Clone + Copy + Send + Sync + 'static + std::fmt::Debug {
    // Aleo
    type Aleo: Network;

    // ETH
    type Sepolia: EthNetwork;
    type Linea: EthNetwork;
    type Zksync: EthNetwork;
    type Goerli: EthNetwork;
    type Scroll: EthNetwork;
    type Optimism: EthNetwork;
    type Arbitrum: EthNetwork;
    type Taiko: EthNetwork;
}

#[macro_export]
macro_rules! eths {
    () => {
        1 | 3 | 4 | 5 | 6 | 7 | 8 | 9
    };
}
