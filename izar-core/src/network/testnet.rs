use aleo_rust::Testnet3;

use super::{eth::EthNetwork, IzarNetwork};

#[derive(Clone, Copy, Debug)]
pub struct IzarTestnet;

impl IzarNetwork for IzarTestnet {
    type Aleo = Testnet3;

    type Sepolia = Sepolia;
    type Linea = LineaTestnet;
    type Zksync = ZksyncTestnet;
    type Goerli = Goerli;
    type Scroll = ScrollTestnet;
    type Optimism = OptimismTestnet;
    type Arbitrum = ArbitrumTestnet;
    type Taiko = TaikoTestnet;
}

#[derive(Clone, Copy, Debug)]
pub struct Sepolia;

impl EthNetwork for Sepolia {
    const WRAPPER_CONTRACT: &'static str = "0x7d164F30f0b6a2ABAE55Adae9645a22268747D61";
    const PROXY_CONTRACT: &'static str = "0xa4DB034df1353F620207AA8ab695318316Fc4D93";
    const LOCK_CONTRACT: &'static str = "0xE5BaBF57e90f9E219a881D24789F742cCAB6f6B1";
    const LOGIC_CONTRACT: &'static str = "0x6221A66723a47890eC66912364F20abd62279365";

    const ETH_CHAIN_ID: u32 = 11155111;
    const IZAR_CHAIN_ID: u32 = 1;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 5468808;
    // const CONFIRMATIONS: usize = 64; // need two epochs
}

#[derive(Clone, Copy, Debug)]
pub struct LineaTestnet;

impl EthNetwork for LineaTestnet {
    const WRAPPER_CONTRACT: &'static str = "0x26f8603fC1Da7D164c3fd353c816c3215Ef807a9";
    const PROXY_CONTRACT: &'static str = "0xd73E1Accb6a22751FC0F6478c79bE83e9E544ac9";
    const LOCK_CONTRACT: &'static str = "0xd06Bcb4556f71cB3035891362A7e3d90e431551a";
    const LOGIC_CONTRACT: &'static str = "0x654dDC5c45C59be6C2699DbCbAd0fA5Bd16F6eC0";

    const ETH_CHAIN_ID: u32 = 59140;
    const IZAR_CHAIN_ID: u32 = 3;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 4044130;
}

#[derive(Clone, Copy, Debug)]
pub struct ZksyncTestnet;

impl EthNetwork for ZksyncTestnet {
    const WRAPPER_CONTRACT: &'static str = "0x89538206cDc690564813687F257bD2f8d2Ad0448";
    const PROXY_CONTRACT: &'static str = "0x3a31dBDffF9Da7cE6215Cf8b42da0654BE3f6E86";
    const LOCK_CONTRACT: &'static str = "0xA5096d0Fd92056Ed99833a201B02E11eF1c82C30";
    const LOGIC_CONTRACT: &'static str = "0x03287Dcf937A1aA78768ef81BA3B0cf6941d2be1";

    const ETH_CHAIN_ID: u32 = 280;
    const IZAR_CHAIN_ID: u32 = 4;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 16748981;
}

#[derive(Clone, Copy, Debug)]
pub struct Goerli;

impl EthNetwork for Goerli {
    const WRAPPER_CONTRACT: &'static str = "0x532e91cA086964251519359271B99Bd08427314f";
    const PROXY_CONTRACT: &'static str = "0xDB249Fda431b6385aD5E028F3AA31f3f51eBAEf2";
    const LOCK_CONTRACT: &'static str = "0x762f1119123806FC0AA4C58f61a9dA096910200B";
    const LOGIC_CONTRACT: &'static str = "0xc3354eE73a00B5d7205a4c0579770984367a9eff";

    const ETH_CHAIN_ID: u32 = 5;
    const IZAR_CHAIN_ID: u32 = 5;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 10612062;
    const CONFIRMATIONS: usize = 32;
}

#[derive(Clone, Copy, Debug)]
pub struct ScrollTestnet;

impl EthNetwork for ScrollTestnet {
    const WRAPPER_CONTRACT: &'static str = "0xdcb58E26413F087312C2BE7D8C1b8B10F90B6C5F";
    const PROXY_CONTRACT: &'static str = "0xDFc105358eFb26E0373741F5ac8d171Ae6897CA7";
    const LOCK_CONTRACT: &'static str = "0xE7f5A4bAA3dd8509E96F26b7920e03965FeFb599";
    const LOGIC_CONTRACT: &'static str = "0xa5A5dC4A6F869e279AC32b1925d2605a96289859";

    const ETH_CHAIN_ID: u32 = 534351;
    const IZAR_CHAIN_ID: u32 = 6;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 3089109;
}

#[derive(Clone, Copy, Debug)]
pub struct OptimismTestnet;

impl EthNetwork for OptimismTestnet {
    const WRAPPER_CONTRACT: &'static str = "0xdcb58E26413F087312C2BE7D8C1b8B10F90B6C5F";
    const PROXY_CONTRACT: &'static str = "0xDFc105358eFb26E0373741F5ac8d171Ae6897CA7";
    const LOCK_CONTRACT: &'static str = "0xE7f5A4bAA3dd8509E96F26b7920e03965FeFb599";
    const LOGIC_CONTRACT: &'static str = "0xa5A5dC4A6F869e279AC32b1925d2605a96289859";

    const ETH_CHAIN_ID: u32 = 420;
    const IZAR_CHAIN_ID: u32 = 7;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 8690339;
}

#[derive(Clone, Copy, Debug)]
pub struct ArbitrumTestnet;

impl EthNetwork for ArbitrumTestnet {
    const WRAPPER_CONTRACT: &'static str = "0xdcb58E26413F087312C2BE7D8C1b8B10F90B6C5F";
    const PROXY_CONTRACT: &'static str = "0xDFc105358eFb26E0373741F5ac8d171Ae6897CA7";
    const LOCK_CONTRACT: &'static str = "0xE7f5A4bAA3dd8509E96F26b7920e03965FeFb599";
    const LOGIC_CONTRACT: &'static str = "0xa5A5dC4A6F869e279AC32b1925d2605a96289859";

    const ETH_CHAIN_ID: u32 = 421613;
    const IZAR_CHAIN_ID: u32 = 8;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 18499678;
}

#[derive(Clone, Copy, Debug)]
pub struct TaikoTestnet;

impl EthNetwork for TaikoTestnet {
    const WRAPPER_CONTRACT: &'static str = "0xdcb58E26413F087312C2BE7D8C1b8B10F90B6C5F";
    const PROXY_CONTRACT: &'static str = "0xDFc105358eFb26E0373741F5ac8d171Ae6897CA7";
    const LOCK_CONTRACT: &'static str = "0xE7f5A4bAA3dd8509E96F26b7920e03965FeFb599";
    const LOGIC_CONTRACT: &'static str = "0xa5A5dC4A6F869e279AC32b1925d2605a96289859";

    const ETH_CHAIN_ID: u32 = 167007;
    const IZAR_CHAIN_ID: u32 = 9;
    const GAS_LIMIT: u128 = 1_000_000;
    const START_HEIGHT: u64 = 100577;
}
