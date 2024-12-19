#![no_std]

use soroban_sdk::{ contract, contractimpl, Address, BytesN, Env, IntoVal, Map, Symbol, Val, Vec };

mod events;
mod crowdfund_client;
mod storage_types;
mod entity;
mod test;
mod testutils;
use storage_types::DataKey;
use events::*;
use entity::*;


fn get_admin(env: &Env) -> Address {
    env.storage().instance().get::<_, Address>(&DataKey::Admin).unwrap()
}

fn get_count(env: &Env) -> u64 {
    env.storage().instance().get::<_, u64>(&DataKey::Counter).unwrap_or(0)
}

fn set_count(env: &Env, counter: u64) {
    env.storage().instance().set(&DataKey::Counter, &counter);
}

fn set_crowdfund_to_map(env: &Env, crowdfund_id: u64, crowdfund_address: Address) {
    let mut crowdfund_map = env.storage().instance().get::<_, Map<u64, Address>>(&DataKey::CrowdfundMap).unwrap();
    crowdfund_map.set(crowdfund_id, crowdfund_address);
    env.storage().instance().set(&DataKey::CrowdfundMap, &crowdfund_map);
}

fn get_crowdfund_from_map(env: &Env, crowdfund_id: u64) -> Address {
    let crowdfund_map = env.storage().instance().get::<_, Map<u64, Address>>(&DataKey::CrowdfundMap).unwrap();
    let address= crowdfund_map.get(crowdfund_id).unwrap();
    address
}

fn creat_crowdfund(env: &Env, crowdfund_args: &CrowdfundArgs) -> u64 {
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_args: Vec<Val> = (crowdfund_args.recipient.clone(), crowdfund_args.deadline.clone(), 
    crowdfund_args.target_amount.clone(), crowdfund_args.token.clone()).into_val(env);
    let deployed_address = crowdfund_client::deploy(&env, crowdfund_args.recipient.clone(), salt, Symbol::new(&env, "initialize"), 
        init_args);
    let mut counter = get_count(&env);
    counter += 1;
    set_crowdfund_to_map(env, counter, deployed_address);
    set_count(&env, counter);
    counter
}

#[contract]
pub struct CrowdfundRegistryContract;

#[contractimpl]
impl CrowdfundRegistryContract {

    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Counter, &0u64);
        env.storage().instance().set(&DataKey::CrowdfundMap, &Map::<u64, Address>::new(&env));
        register_init_event(&env, admin);
    }


    
    pub fn creat_batch_crowdfunds(env: Env, crowdfund_args_list: Vec<CrowdfundArgs>) -> u64 {
        let admin= get_admin(&env);
        admin.require_auth();
        for crowdfund_args in crowdfund_args_list.iter() {
            let crowdfund_id = creat_crowdfund(&env, &crowdfund_args);
            creat_crowdfund_event(&env, crowdfund_id, crowdfund_args.recipient.clone(), 
                crowdfund_args.deadline.clone(), crowdfund_args.target_amount.clone(), crowdfund_args.token.clone());
        }
        let counter= get_count(&env);
        counter
    }

    pub fn deposit_to_batch_crowdfunds(env: Env, batch_pledge: Vec<Pledge>) {
        // Get the current count.
        let admin= get_admin(&env);
        admin.require_auth();
        for pledge in batch_pledge.iter() {
            let crowdfund_address= get_crowdfund_from_map(&env, pledge.crowdfund_id.clone());
            crowdfund_client::deposit(&env, crowdfund_address.clone(), pledge.donor.clone(), pledge.amount.clone());
        }
        events::deposit_batch_crowdfunds_event(&env, batch_pledge);

    }

    pub fn withdraw_from_batch_crowdfunds(env: Env, batch_withdraw: Vec<Withdraw>) {
        // Get the current count.
        let admin= get_admin(&env);
        admin.require_auth();
        for withdraw in batch_withdraw.iter() {
            let crowdfund_address= get_crowdfund_from_map(&env, withdraw.crowdfund_id.clone());
            crowdfund_client::withdraw(&env, crowdfund_address.clone(), withdraw.drawer.clone());
        }
        events::withdraw_batch_crowdfunds_event(&env, batch_withdraw);

    }
}




