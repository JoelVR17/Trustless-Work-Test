use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec, Val};
use soroban_sdk::token::{self, Interface as _};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::admin::{has_administrator, write_administrator, read_administrator};
use crate::allowance::{read_allowance, write_allowance, spend_allowance};
use crate::balance::receive_balance;
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::metadata::write_metadata;
use soroban_token_sdk::TokenUtils;

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount);
    }
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&e) {
            panic!("already initialized");
        }
        write_administrator(&e, &admin);
        if decimal > 18 {
            panic!("Decimal must not be greater than 18");
        }
    
        let metadata = TokenMetadata {
            decimal,
            name: name.clone(),
            symbol: symbol.clone(),
        };
    
        write_metadata(&e, metadata);
    
        e.events().publish((symbol_short!("init"),), Vec::<Val>::new(&e));
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
        
        // let balance: i128 = e.storage().instance().get(&to).unwrap_or(0);
        // e.storage().instance().set(&to, &(balance + amount));
    
        // e.events().publish(
        //     (symbol_short!("mint"), to),
        //     amount
        // );
    }
}

#[contractimpl]
impl token::Interface for Token {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        // Implementa la lógica de 'allowance' usando `read_allowance` de allowance.rs
        let allowance_value = read_allowance(&e, from.clone(), spender.clone());
        allowance_value.amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        check_nonnegative_amount(amount);
        write_allowance(&e, from.clone(), spender, amount, expiration_ledger);
    }    

    fn balance(e: Env, id: Address) -> i128 {
        e.storage().instance().get(&id).unwrap_or(0)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        let from_balance: i128 = e.storage().instance().get(&from).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        let to_balance: i128 = e.storage().instance().get(&to).unwrap_or(0);
        e.storage().instance().set(&from, &(from_balance - amount));
        e.storage().instance().set(&to, &(to_balance + amount));
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        // Lógica para verificar y reducir el allowance usando spend_allowance
        spend_allowance(&e, from.clone(), spender.clone(), amount);

        // Reducir el balance y transferir
        let from_balance: i128 = e.storage().instance().get(&from).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        let to_balance: i128 = e.storage().instance().get(&to).unwrap_or(0);
        e.storage().instance().set(&from, &(from_balance - amount));
        e.storage().instance().set(&to, &(to_balance + amount));
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        let from_balance: i128 = e.storage().instance().get(&from).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        e.storage().instance().set(&from, &(from_balance - amount));
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        // Verificar y reducir allowance usando spend_allowance
        spend_allowance(&e, from.clone(), spender.clone(), amount);

        // Reducir el balance
        let from_balance: i128 = e.storage().instance().get(&from).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        e.storage().instance().set(&from, &(from_balance - amount));
    }

    fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&"decimal").unwrap_or(0)
    }

    fn name(e: Env) -> String {
        e.storage().instance().get(&"name").unwrap()
    }

    fn symbol(e: Env) -> String {
        e.storage().instance().get(&"symbol").unwrap()
    }
}
