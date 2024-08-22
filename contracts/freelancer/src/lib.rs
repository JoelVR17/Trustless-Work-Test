#![no_std]

mod storage;
mod storage_types;
mod allowance;
mod balance;
mod admin;
mod metadata;
mod token;
mod events;
mod contract;
mod test;

pub use crate::contract::FreelanceContractClient;