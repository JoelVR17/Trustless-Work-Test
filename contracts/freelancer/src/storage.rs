use core::ptr::null;

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec, symbol_short};

use crate::storage_types::Project;

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

pub fn get_project(e: &Env, project_id: u128) -> (Project, Symbol) {
    // Obtener la clave del proyecto
    let mut key_bytes = [0u8; 32];
    let prefix_bytes = b"project_";
    key_bytes[..8].copy_from_slice(prefix_bytes);
    key_bytes[8..16].copy_from_slice(&project_id.to_le_bytes());
    let key_str = core::str::from_utf8(&key_bytes).unwrap();
    let project_key = Symbol::new(&e, key_str);

    // Obtener el proyecto del almacenamiento
    let project: Project = e.storage().instance().get(&project_key).unwrap();

    // Return the project and the key
    (project, project_key)
}













// ! ESTAS 3 FUNCIONES ES PARA OBTENER LOS KEYS DE LOS PROYCTOS QUE SE GUARDAN, CUANDO YA SE TIENEN, SE ITERA PARA CREAR UN ARRAY CON TODOS LOS PROYECTOS Y RETORNARLOS A LA FUNCION `get_projects_by_freelancer` -> AUN NO SIRVE, PERO HAY QUE SEGUIR TRABAJANDO EN ELLO
pub fn get_all_projects(e: &Env) -> Vec<Project> {
    let mut projects: Vec<Project> = Vec::new(&e);
    
    let keys = get_project_keys(e);
    
    // Itera sobre las claves con un índice.
    for (i, key) in keys.iter().enumerate() { 

        if let Some(project) = e.storage().persistent().get::<_, Project>(&key) {
            projects.set(i as u32, project);
        }
    }
    
    projects  // Devuelve el vector de proyectos.
}

// Funciones para gestionar la lista de claves
pub fn get_project_keys(e: &Env) -> Vec<Symbol> {
    let mut keys: Vec<Symbol> = Vec::new(e);
    
    let total_projects: u32 = e
        .storage()
        .persistent()
        .get::<_, u32>(&Symbol::symbol_short!("total_projects"))
        .unwrap_or(0);

    for i in 0..total_projects {
        let key = Symbol::symbol_short!(&format!("project_{}", i));
        keys.push(key);
    }

    keys
}

pub fn set_project_keys(e: &Env, keys: &[Symbol]) {

    let key = Symbol::new(&e,"project_keys");
    e.storage().persistent().set(&key, &keys.to_vec());
}

// pub fn get_total_address(e: &Env) -> u32 {
//     e.storage().instance().get(&DataKey::TotalAddress).unwrap()
// }