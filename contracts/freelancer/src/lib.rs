#![no_std]
use soroban_sdk::{
    contract, contractimpl, Address, Env, Symbol, Vec, Map, contracttype, symbol_short
};

mod storage;

#[contracttype]
pub struct Objective {
    pub price: u128,
    pub half_paid: u128,
    pub completed: bool,
}

#[contracttype]
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

use storage::get_address;

#[contract]
pub struct FreelanceContract;

#[contractimpl]
impl FreelanceContract {
    pub fn create_project(
        e: Env,
        freelancer: Address,
        prices: Vec<u128>
    ) -> u128 {
        let contract_key = symbol_short!("p_count");

        let mut project_count: u128 = e
            .storage()
            .instance()
            .get(&contract_key)
            .unwrap_or(0);

        project_count += 1;
        e.storage().instance().set(&contract_key, &project_count);

        let mut objectives: Map<u128, Objective> = Map::new(&e);
        for (i, price) in prices.iter().enumerate() {
            objectives.set(i as u128, Objective {
                price: price,
                half_paid: 0,
                completed: false,
            });
        }

        let project = Project {
            client: get_address(&e, 0),
            freelancer,
            objectives_count: prices.len() as u128,
            objectives,
            completed_objectives: 0,
            earned_amount: 0,
            contract_balance: 0,
            cancelled: false,
            completed: false,
        };

        let mut key_bytes = [0u8; 32];
        let prefix_bytes = b"project_";
        key_bytes[..8].copy_from_slice(prefix_bytes);
        key_bytes[8..16].copy_from_slice(&project_count.to_le_bytes());

        let key_str = core::str::from_utf8(&key_bytes).unwrap();

        let project_key = Symbol::new(&e, key_str);
        e.storage().instance().set(&project_key, &project);

        project_count
    }

    pub fn fund_objective(e: Env, project_id: u128, objective_id: u128, usdc_token: Address) {
        // Obtener la clave del proyecto
        let mut key_bytes = [0u8; 32];
        let prefix_bytes = b"project_";
        key_bytes[..8].copy_from_slice(prefix_bytes);
        key_bytes[8..16].copy_from_slice(&project_id.to_le_bytes());
        let key_str = core::str::from_utf8(&key_bytes).unwrap();
        let project_key = Symbol::new(&e, key_str);

        // Obtener el proyecto del almacenamiento
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();

        // Verificar que la persona que invoca la función es el cliente
        let invoker = get_address(&e, 0);
        if invoker != project.client {
            panic!("Only the client can fund objectives");
        }

        // Obtener el objetivo del proyecto
        let mut objective = project.objectives.get(objective_id).unwrap();

        // Verificar que el objetivo no ha sido financiado previamente
        if objective.half_paid > 0 {
            panic!("Objective already funded");
        }

        // Calcular la mitad del precio del objetivo y convertirlo a i128
        let half_price = (objective.price / 2) as i128;

        // Transferir la mitad del precio desde el cliente al contrato
        // Para Stellar, utilizamos el contrato de USDC (aquí simplificado)
        let usdc_client = soroban_sdk::token::Client::new(&e, &usdc_token);
        usdc_client.transfer_from(
            &invoker,  
            &project.client,
            &e.current_contract_address(),
            &half_price       
        );

        // Actualizar el objetivo para reflejar el pago parcial
        objective.half_paid = half_price as u128;
        project.objectives.set(objective_id, objective);

        // Guardar el proyecto actualizado
        e.storage().instance().set(&project_key, &project);
    }

    pub fn complete_objective(e: Env, project_id: u128, objective_id: u128, usdc_token: Address) {
        // Obtener la clave del proyecto
        let mut key_bytes = [0u8; 32];
        let prefix_bytes = b"project_";
        key_bytes[..8].copy_from_slice(prefix_bytes);
        key_bytes[8..16].copy_from_slice(&project_id.to_le_bytes());
        let key_str = core::str::from_utf8(&key_bytes).unwrap();
        let project_key = Symbol::new(&e, key_str);

        // Obtener el proyecto del almacenamiento
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();

        // Verificar que la persona que invoca la función es el freelancer
        let invoker = get_address(&e, 0);
        if invoker != project.freelancer {
            panic!("Only the freelancer can complete objectives");
        }

        // Obtener el objetivo del proyecto
        let mut objective = project.objectives.get(objective_id).unwrap();

        // Verificar que el objetivo ha sido financiado parcialmente
        if objective.half_paid == 0 {
            panic!("Objective not funded");
        }

        // Verificar que el objetivo no ha sido completado previamente
        if objective.completed {
            panic!("Objective already completed");
        }

        // Calcular el precio restante del objetivo
        let remaining_price = (objective.price - objective.half_paid) as i128;

        // Transferir el precio restante desde el cliente al contrato
        let usdc_client = soroban_sdk::token::Client::new(&e, &usdc_token);
        usdc_client.transfer_from(
            &project.client,  
            &project.client, // La cuenta fuente es el cliente
            &e.current_contract_address(), // El contrato es el receptor
            &remaining_price
        );

        // Transferir el precio total del objetivo al freelancer
        usdc_client.transfer(
            &e.current_contract_address(), // El contrato transfiere los fondos
            &project.freelancer,           // El freelancer es el receptor
            &(objective.price as i128)     // El precio total del objetivo
        );

        // Marcar el objetivo como completado y actualizar los contadores
        objective.completed = true;
        project.completed_objectives += 1;
        project.earned_amount += objective.price;

        // Actualizar el objetivo en el almacenamiento
        project.objectives.set(objective_id, objective);

        // Guardar el proyecto actualizado
        e.storage().instance().set(&project_key, &project);
    }
}

#[cfg(test)]
mod test;