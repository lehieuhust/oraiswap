pub mod contract;
mod math;
mod migration;
mod querier;
mod rewards;
mod staking;
mod state;

#[cfg(all(target_arch = "wasm32", not(feature = "library")))]
cosmwasm_std::create_entry_points!(contract);

#[cfg(test)]
mod testing;