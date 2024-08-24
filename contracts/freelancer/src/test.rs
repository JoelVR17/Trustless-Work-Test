#![cfg(test)]

extern crate std;

use crate::storage_types::{Project, DataKey};
use crate::{contract::FreelanceContract, FreelanceContractClient};
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env, Vec, IntoVal, symbol_short, log};
use crate::token::{ Token, TokenClient };

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test_create_fund_complete_objectives() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);

    let usdc_contract_id = env.register_contract(None, Token {});
    let token = create_token(&env, &admin1);
    
    let contract_address = env.register_contract(None, FreelanceContract); 
    let freelance_client = FreelanceContractClient::new(&env, &contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let expiration_ledger = env.ledger().sequence() + 1000;
    let full_price = 100;
    token.approve(&client_address, &contract_address, &full_price, &expiration_ledger);

    let prices = Vec::from_array(&env, [100_u128, 100_u128]);
    let project_id = freelance_client.create_project(&freelancer_address, &prices, &client_address);

    freelance_client.fund_objective(&project_id, &0, &client_address, &usdc_contract_id, &contract_address);

    let project_key = DataKey::Project(project_id);
    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let first_objective = project.objectives.get(0).unwrap();
    assert_eq!(first_objective.half_paid, 50);

    freelance_client.complete_objective(&project_id, &0, &freelancer_address, &usdc_contract_id);

    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let first_objective = project.objectives.get(0).unwrap();
    assert_eq!(first_objective.completed, true);

    freelance_client.fund_objective(&project_id, &1, &client_address, &usdc_contract_id, &contract_address);
    freelance_client.complete_objective(&project_id, &1, &freelancer_address, &usdc_contract_id);

    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let second_objective = project.objectives.get(1).unwrap();
    assert_eq!(second_objective.completed, true);
}

/*
#[test]
fn test_token_functions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    // Register a Stellar asset contract (token contract) in the test environment
    let token_contract_id = env.register_stellar_asset_contract(admin.clone());
    
    // Register your contract
    let contract_id = env.register_contract(None, YourContract);
    let client = YourContractClient::new(&env, &contract_id);

    // Now you can test your contract's interactions with the token contract
    // ...
}
*/