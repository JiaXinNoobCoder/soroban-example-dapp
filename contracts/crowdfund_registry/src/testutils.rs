#![cfg(test)]

use crate::CrowdfundRegistryContractClient;

use soroban_sdk::{Address, Env};

pub fn register_test_contract(e: &Env) -> Address {
    e.register_contract(None, crate::CrowdfundRegistryContract {})
}

pub struct CrowdfundRegistryContract {
    env: Env,
    contract_id: Address,
}

impl CrowdfundRegistryContract {
    #[must_use]
    pub fn client(&self) -> CrowdfundRegistryContractClient {
        CrowdfundRegistryContractClient::new(&self.env, &self.contract_id)
    }

    #[must_use]
    pub fn new(env: &Env, contract_id: Address) -> Self {
        Self {
            env: env.clone(),
            contract_id,
        }
    }
}
