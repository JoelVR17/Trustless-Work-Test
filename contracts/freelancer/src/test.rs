#![cfg(test)]

use crate::storage_types::Project;
use crate::{contract::FreelanceContract, FreelanceContractClient};
use soroban_sdk::{ testutils::Address as _,Address, Env, Symbol, Vec};

#[test]
fn test_create_fund_complete_objectives() {
    let env = Env::default();

    env.mock_all_auths();
    
    // Generar direcciones para el cliente y freelancer usando `random`
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);

    // Registrar el contrato
    let contract_id: Address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &contract_id);

    // Crear un proyecto
    let prices = Vec::from_array(&env, [100u128, 100u128]);
    freelance_client.create_project(&freelancer_address, &prices, &client_address);

    // Cliente financia el primer objetivo
    freelance_client.fund_objective(&1u128, &0u128, &client_address);

    // Verificar que el objetivo ha sido parcialmente financiado
    // En lugar de usar `get_project`, podemos verificar el almacenamiento directamente.
    let project_key = Symbol::new(&env, "project_1");
    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let first_objective = project.objectives.get(0).unwrap();
    assert_eq!(first_objective.half_paid, 50u128);

    // Freelancer completa el primer objetivo
    freelance_client.complete_objective(&1u128, &0u128, &freelancer_address);

    // Verificar que el primer objetivo ha sido completado
    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let first_objective = project.objectives.get(0).unwrap();
    assert_eq!(first_objective.completed, true);

    // Cliente financia el segundo objetivo
    freelance_client.fund_objective(&1u128, &1u128, &client_address);

    // Freelancer completa el segundo objetivo
    freelance_client.complete_objective(&1u128, &1u128, &freelancer_address);

    // Verificar que el segundo objetivo ha sido completado
    let project: Project = env.storage().instance().get(&project_key).unwrap();
    let second_objective = project.objectives.get(1).unwrap();
    assert_eq!(second_objective.completed, true);
}