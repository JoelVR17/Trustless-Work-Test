use soroban_sdk::{
    contract, contractimpl, Address, Env, Vec, Map, symbol_short, String
};

use crate::storage::{get_project, get_all_projects};
use crate::storage_types::{Objective, Project, User, DataKey};
use crate::token::TokenClient;
use crate::events::{project_created, objective_added, objective_completed, objective_funded, project_cancelled, project_completed, project_refunded};

#[contract]
pub struct FreelanceContract;

#[contractimpl]
impl FreelanceContract {

    pub fn create_project(
        e: Env,
        freelancer: Address,
        prices: Vec<u128>,
        user: Address,
    ) -> u128 {
        user.require_auth();  

        // Ensure prices are not empty
        if prices.is_empty() {
            panic!("Prices cannot be empty");
        }
        
        let contract_key = symbol_short!("p");
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
            client: user.clone(),
            freelancer: freelancer.clone(),
            objectives_count: prices.len() as u128,
            objectives,
            completed_objectives: 0,
            earned_amount: 0,
            contract_balance: 0,
            cancelled: false,
            completed: false,
        };
    
        let project_key = DataKey::Project(project_count);
        e.storage().instance().set(&project_key, &project);
    
        // Emitir el evento del proyecto creado
        project_created(&e, project_key, user.clone(), freelancer.clone(), prices);
    
        project_count
    }

    pub fn complete_project(e: Env, project_id: u128, user: Address) {

        // Obtener el proyecto
        let (mut project, project_key) = get_project(&e, project_id);

        // Verificar que la persona que invoca la función es el cliente
        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        // Check if the project is cancelled
        if !project.completed {
            panic!("Project is completed");
        }

        // Check if the project is cancelled
        if !project.cancelled {
            panic!("Project is cancelled");
        }

        // Check if all the objectives are completed
        if project.completed_objectives == project.objectives_count {
            panic!("Not all objectives completed");
        }

        // Now, the project is completed
        project.completed = true;

        // Save project
        e.storage().instance().set(&project_key, &project);
    
        // Emitir el evento con el ID del project
        project_completed(&e, project_key);

    }

    pub fn complete_objective(
        e: Env,
        project_id: u128,
        objective_id: u128,
        user: Address,
        usdc_contract: Address // Se usa el contrato de USDC
    ) {
        user.require_auth();
    
        let project_key = DataKey::Project(project_id);
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();
    
        // Verificar que el usuario es el freelancer
        if user != project.freelancer {
            panic!("Only the freelancer can complete objectives");
        }
    
        let mut objective = project.objectives.get(objective_id).unwrap();
    
        // Verificar que el objetivo ha sido parcialmente financiado
        if objective.half_paid == 0 {
            panic!("Objective not funded");
        }
    
        // Verificar que el objetivo no ha sido completado previamente
        if objective.completed {
            panic!("Objective already completed");
        }
    
        let remaining_price = (objective.price - objective.half_paid) as i128;
        let full_price = objective.price;
    
        // Transferencia del cliente al contrato para el precio restante
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        usdc_client.transfer_from(
            &project.client,
            &user, // spender
            &e.current_contract_address(),
            &remaining_price
        );
    
        // Transferencia del contrato al freelancer del precio total del objetivo
        usdc_client.transfer(
            &e.current_contract_address(),
            &project.freelancer,
            &(objective.price as i128)
        );
    
        // Marcar el objetivo como completado
        objective.completed = true;
        project.completed_objectives += 1;
        project.earned_amount += objective.price;
    
        // Guardar el proyecto actualizado
        project.objectives.set(objective_id, objective);
        e.storage().instance().set(&project_key, &project);
    
        // Emitir el evento de objetivo completado
        objective_completed(&e, project_key, objective_id, full_price);
    }

    pub fn cancel_project(e: Env, project_id: u128, user: Address) {
        user.require_auth();
        // Obtener el proyecto
        let (mut project, project_key) = get_project(&e, project_id);

        // Verificar que la persona que invoca la función es el cliente
        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        // Check if the project is completed
        if !project.completed {
            panic!("Project is completed");
        }

        // Check if the project is cancelled
        if !project.cancelled {
            panic!("Project is cancelled");
        }

        // Now, the project is cancelled
        project.cancelled = true;

        // Save project
        e.storage().instance().set(&project_key, &project);

         // Emitir el evento con el ID del project
         project_cancelled(&e, project_key);
    }

    pub fn add_objective(e: Env, project_id: u128, prices: Vec<u128>, user: Address) {
        user.require_auth();
        // Obtener el proyecto
        let (mut project, project_key) = get_project(&e, project_id);

        // Verificar que la persona que invoca la función es el cliente
        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can add objectives");
        }

        // Check if the project is cancelled
        if !project.completed {
            panic!("Project is completed");
        }

        // Check if the project is cancelled
        if !project.cancelled {
            panic!("Project is cancelled");
        }
        
         // Iterar sobre los precios y agregar objetivos
        for (i, price) in prices.iter().enumerate() {
            let objective_id = project.objectives_count + i as u128;

            project.objectives.set(objective_id, Objective {
                price: price,
                half_paid: 0,
                completed: false,
            });

            // Emitir el evento con el ID del objetivo
            objective_added(&e, &project_key, objective_id, price);
        }

        // Actualizar el recuento de objetivos del proyecto
        project.objectives_count += prices.len() as u128;

        // Guardar el proyecto actualizado en el almacenamiento
        e.storage().instance().set(&project_key, &project);
    }

    pub fn fund_objective(e: Env, project_id: u128, objective_id: u128, user: Address, usdc_contract: Address) {
        user.require_auth();
    
        let project_key = DataKey::Project(project_id);
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();
    
        if user != project.client {
            panic!("Only the client can fund objectives");
        }
    
        let mut objective = project.objectives.get(objective_id).unwrap();
        if objective.half_paid > 0 {
            panic!("Objective already funded");
        }
    
        let half_price = (objective.price / 2) as i128;
        let usdc_client = TokenClient::new(&e, &usdc_contract);
    
        usdc_client.transfer_from(
            &user,  
            &user,         
            &e.current_contract_address(), 
            &half_price             
        );
    
        objective.half_paid = half_price as u128;
        project.objectives.set(objective_id, objective);
        e.storage().instance().set(&project_key, &project);
    
        objective_funded(&e, project_key, objective_id, half_price as u128);
    }

    pub fn refund_remaining_funds(e: Env, project_id: u128, objective_id: u128, user: Address) {
        user.require_auth();
        // Obtener el proyecto
        let (mut project, project_key) = get_project(&e, project_id);

        // Verificar que la persona que invoca la función es el cliente
        let invoker = user.clone();
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        // Check if the project is cancelled
        if !project.cancelled {
            panic!("Project is cancelled");
        }


        let mut refundableAmount : i128 = 0;

        for i in 0..project.objectives_count {
            // Obtener el objetivo del proyecto
            let mut objective = project.objectives.get(objective_id).unwrap(); 


            //Objective storage objective = project.objectives[i];
            if !objective.completed && objective.half_paid > 0 {
                refundableAmount += objective.half_paid as i128;
                objective.half_paid = 0; // Prevent double refund
            }
        }

        let usdc_client = soroban_sdk::token::Client::new(&e, &user);
        let mut contract_balance = usdc_client.balance(&e.current_contract_address());


        // Determinar si el contrato tiene fondos suficientes 
        if  contract_balance >= refundableAmount {
            panic!("Insufficient contract balance");
        }

        // Transferir el precio total del objetivo al freelancer
        usdc_client.transfer(
            &e.current_contract_address(), // El contrato transfiere los fondos
            &project.client,           // El freelancer es el receptor
            &(refundableAmount as i128)     // El precio total del objetivo
        );

        project_refunded(&e, project_key, user.clone(), refundableAmount as u128);

    }
    
    pub fn get_projects_by_freelancer(e: Env, freelancer: Address) -> Vec<Project> {
        
        // Obtener todos los proyectos
        let all_projects: Vec<Project> = get_all_projects(e.clone());

        // Crear un vector para almacenar los proyectos que pertenecen al freelancer
        let mut result: Vec<Project> = Vec::new(&e);
        let mut index: u32 = 0;

        for i in 0..all_projects.len() {
            // Obtener el proyecto por su índice en el vector
            let project = all_projects.get(i).unwrap(); // Aquí `i` es el índice en el vector

            // Verificar si el proyecto pertenece al freelancer
            if project.freelancer == freelancer {
                result.set(index, project); // Añadir el proyecto al vector resultado
                index += 1;
            }
        }

        result
    }

    
    pub fn register(e: Env, user_address: Address, name: String, email: String) -> bool {
        user_address.require_auth();

        let key = DataKey::User(user_address.clone());

        // Check if user already exists
        if e.storage().persistent().has(&key) {
            return false; // User already registered
        }

        let user_id = e
            .storage()
            .persistent()
            .get(&DataKey::UserCounter)
            .unwrap_or(0)
            + 1;

        e.storage()
            .persistent()
            .set(&DataKey::UserCounter, &user_id);

        let user = User {
            id: user_id,
            user: user_address.clone(),
            name: name.clone(),
            email: email.clone(),
            registered: true,
            timestamp: e.ledger().timestamp(),
        };

        e.storage()
            .persistent()
            .set(&DataKey::User(user_address.clone()), &user);

        // Generate a unique transaction ID (using the current ledger sequence number)
        let user_reg_id = e.ledger().sequence();

        // Store the transaction ID
        e.storage()
            .persistent()
            .set(&DataKey::UserRegId(user_address.clone()), &user_reg_id);

        //more checks to know if user successfully registered
        return true;
    }

    pub fn login(e: Env, user_address: Address) -> String {
        user_address.require_auth();
    
        let key = DataKey::User(user_address.clone());
    
        if let Some(user) = e.storage().persistent().get::<_, User>(&key) {
            user.name
        } else {
            soroban_sdk::String::from_str(&e, "User not found")
        }
    }

}