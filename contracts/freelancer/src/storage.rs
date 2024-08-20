use soroban_sdk::{contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]

enum DataKey {
    Initialized,
    TotalAddress,
    Shares(u32),
    Addresses(u32),
}

pub fn get_address(e: &Env, index: u32) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Addresses(index))
        .unwrap()
}

// pub fn get_total_address(e: &Env) -> u32 {
//     e.storage().instance().get(&DataKey::TotalAddress).unwrap()
// }