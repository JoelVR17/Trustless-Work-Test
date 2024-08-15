#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec, Address, Map};

pub trait IERC20Token {
    fn transfer_from(env: Env, sender: Address, recipient: Address, amount: u128) -> bool;
    fn transfer(env: Env, recipient: Address, amount: u128) -> bool;
    fn balance_of(env: Env, account: Address) -> u128;
}

#[derive(Clone)]
pub struct Objective {
    pub price: u128,
    pub half_paid: u128,
    pub completed: bool,
}

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

#[derive(Clone)]
pub struct ProjectInfo {
    pub client: Address,
    pub freelancer: Address,
    pub objectives_count: u128,
    pub completed_objectives: u128,
    pub earned_amount: u128,
    pub contract_balance: u128,
    pub cancelled: bool,
    pub completed: bool,
}

#[contract]
pub struct FreelanceContract {
    pub usdc_token: Address,
    pub owner: Address,
    pub projects: Map<u128, Project>,
    pub project_count: u128,
}

#[contractimpl]
impl FreelanceContract {

    pub fn create_project(
        env: Env,
        freelancer: Address,
        prices: Vec<u128>
    ) -> u128 {
        let mut contract = FreelanceContract::from_env(env); // ! error here
        
        // Increment project count
        contract.project_count += 1;
        let project_id = contract.project_count;
        
        // Create a new project
        let mut project = Project {
            client: get_sender(env),
            freelancer: freelancer,
            objectives_count: prices.len() as u128,
            objectives: Vec::new(), // ! error here
            completed_objectives: 0,
            earned_amount: 0,
            contract_balance: 0,
            cancelled: false,
            completed: false,
        };

        // Add objectives to the project
        for (i, price) in prices.iter().enumerate() {
            let objective = Objective {
                price: *price, // ! error here
                half_paid: 0,
                completed: false,
            };
            project.objectives.insert(i as u128, objective);  // ! error here
        }
        
        // Store the project in the contract
        contract.projects.insert(project_id, project);
        
        // Return project count
        project_id
    }
}

// Get the sender through Env
pub fn get_sender(env: Env) -> Address {
    let sender = env.current_contract_address();
    sender
}

mod test;
