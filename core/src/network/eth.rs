pub trait EthNetwork: Send + Sync + Clone + 'static + std::fmt::Debug {
    const WRAPPER_CONTRACT: &'static str;
    const PROXY_CONTRACT: &'static str;
    const LOCK_CONTRACT: &'static str;
    const LOGIC_CONTRACT: &'static str;

    const ETH_CHAIN_ID: u32;
    const IZAR_CHAIN_ID: u32;
    const GAS_LIMIT: u128;
    const START_HEIGHT: u64;

    const REQUEST_BLOCK_NUM: usize = 1000;

    const CONFIRMATIONS: usize = 0; // L1 need 12 confirmations, L2 need 0 confirmations

    fn format_str<T: std::fmt::LowerHex>(t: T) -> String {
        format!("{:#020x}", t)
    }
}
