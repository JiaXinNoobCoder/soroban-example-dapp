#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol, Val, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrowdfundArgs {
    recipient: Address,
    deadline: u64,
    target_amount: i128,
    token: Address
}

#[contract]
pub struct CrowdfundRegistryContract;

#[contractimpl]
impl CrowdfundRegistryContract {
    
    pub fn creat(env: Env, incr: u32) -> u32 {
        // Get the current count.
    }
}



