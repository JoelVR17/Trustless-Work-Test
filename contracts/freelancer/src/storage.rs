use soroban_sdk::{contracttype, Env, symbol_short, Vec};

use crate::storage_types::{ Project, DataKey };

#[derive(Clone)]
#[contracttype]

enum DataKeyAddress {
    Initialized,
    TotalAddress,
    Shares(u32),
    Addresses(u32),
}

pub fn get_project(e: &Env, project_id: u128) -> (Project, DataKey) {
    // Obtener la clave del proyecto
    let project_key = DataKey::Project(project_id);

    // Obtener el proyecto del almacenamiento
    let project: Project = e.storage().instance().get(&project_key).unwrap();

    // Return the project and the key
    (project, project_key)
}

pub fn get_all_projects(e: Env) -> Vec<Project> {
    let project_count: u128 = e
        .storage()
        .instance()
        .get(&symbol_short!("p_count"))
        .unwrap_or(0);

    let mut projects: Vec<Project> = Vec::new(&e);

    for id in 1..=project_count {
        let project_key = DataKey::Project(id);
        if let Some(project) = e.storage().instance().get(&project_key) {
            projects.set(id as u32,project);
        }
    }

    projects
}

pub fn get_total_address(e: &Env) -> u32 {
    e.storage().instance().get(&DataKeyAddress::TotalAddress).unwrap()
}