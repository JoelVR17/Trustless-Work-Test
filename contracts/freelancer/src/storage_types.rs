use soroban_sdk::{contracttype, Address, Map};

#[contracttype]
#[derive(Clone)]
pub struct Objective {
    pub price: u128,
    pub half_paid: u128,
    pub completed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Project {
    pub client: Address,
    pub freelancer: Address,
    pub objectives_count: u128,
    pub objectives: Map<u128, Objective>,
    pub completed_objectives: u128,
    pub earned_amount: u128,
    pub contract_balance: u128,
    pub cancelled: bool,
    pub completed: bool,
}