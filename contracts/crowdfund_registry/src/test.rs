#![cfg(test)]

use super::testutils::{register_test_contract as register_crowdfund_registry, CrowdfundRegistryContract};
use super::storage_types::DataKey;
use super::entity::*;
use soroban_sdk::testutils::Events;
use soroban_sdk::{ IntoVal, vec, Env, Map, testutils::Address as TestAddress, Address, Vec, Val, Symbol };

fn create_crowdfund_registry_contract(e: &Env) -> (Address, CrowdfundRegistryContract) {
    let contract_id = register_crowdfund_registry(e);
    let crowdfund_registry = CrowdfundRegistryContract::new(e, contract_id.clone());
    (contract_id, crowdfund_registry)
}

fn generate_crowdfund_args_list(env: &Env) -> Vec<CrowdfundArgs> {
    vec![env,
        CrowdfundArgs {
            creator: Address::generate(env),
            recipient: Address::generate(env),
            deadline: 1000,
            target_amount: 5000,
            token: Address::generate(env),
        },
        CrowdfundArgs {
            creator: Address::generate(env),
            recipient: Address::generate(env),
            deadline: 2000,
            target_amount: 10000,
            token: Address::generate(env),
        },
        CrowdfundArgs {
            creator: Address::generate(env),
            recipient: Address::generate(env),
            deadline: 3000,
            target_amount: 15000,
            token: Address::generate(env),
        },
    ]
}

fn get_admin(env: &Env, contract_id: &Address) -> Address {
    let admin = env.as_contract(contract_id, || {
        env.storage().instance().get::<_, Address>(&DataKey::Admin).unwrap()
    });
    admin
}

fn get_counter(env: &Env, contract_id: &Address) -> u64 {
    let counter = env.as_contract(contract_id, || {
        env.storage().instance().get::<_, u64>(&DataKey::Counter).unwrap()
    });
    counter
}

fn get_map(env: &Env, contract_id: &Address) -> Map<u64, Address> {
    let map = env.as_contract(contract_id, || {
        env.storage().instance().get::<_, Map<u64, Address>>(&DataKey::CrowdfundMap).unwrap()
    });
    map
    
}

struct Setup {
    env: Env,
    registry_admin: Address,
    crowdfund_registry: CrowdfundRegistryContract,
    crowdfund_registry_id: Address,
    crowdfund_args_list: Vec<CrowdfundArgs>
}

impl Setup {
    fn new() -> Self {
        let env = Env::default();
        let registry_admin = Address::generate(&env);
        let (crowdfund_registry_id, crowdfund_registry) = 
            create_crowdfund_registry_contract(&env);
        let crowdfund_args_list = generate_crowdfund_args_list(&env);
        Self {
            env,
            registry_admin,
            crowdfund_registry,
            crowdfund_registry_id,
            crowdfund_args_list,
        }
    }
}

#[test]
fn test_initialize() {
    // Call the initialize function
    let setup= Setup::new();
    let env = &setup.env;
    setup.
      crowdfund_registry.
      client().
      initialize(&setup.registry_admin);

     // Verify that the admin was set correctly
     let stored_admin = get_admin(env, &setup.crowdfund_registry_id);
     assert_eq!(stored_admin, setup.registry_admin);

     // Verify that the counter was set to 0
     let counter = get_counter(env, &setup.crowdfund_registry_id);
     assert_eq!(counter, 0);

     // Verify that the crowdfund map was initialized
     let crowdfund_map: Map<u64, Address> = get_map(env, &setup.crowdfund_registry_id);
     assert!(crowdfund_map.is_empty());
     let mut all_events: Vec<(Address, Vec<Val>, Val)> = vec![&setup.env];
     setup
       .env
       .events()
       .all()
       .iter()
       .filter(|event| event.0 == setup.crowdfund_registry_id)
       .for_each(|event| all_events.push_back(event));
    assert_eq!(
        all_events,
        vec![
            &setup.env,
            (
                setup.crowdfund_registry_id,
                (Symbol::new(&setup.env, "registry_init_success"),).into_val(&setup.env),
                setup.registry_admin.into_val(&setup.env)
            )
        ]
    );


}

#[test]
fn test_creat_batch_crowdfunds() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&setup.crowdfund_args_list);
    

}



