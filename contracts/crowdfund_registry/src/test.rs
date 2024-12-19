#![cfg(test)]

extern crate std;
use super::testutils::{register_test_contract as register_crowdfund_registry, CrowdfundRegistryContract};
use super::storage_types::DataKey;
use super::entity::*;
use soroban_sdk::testutils::Events;
use soroban_sdk::{ token, IntoVal, vec, Env, Map, testutils::Address as TestAddress, Address, Vec, Val, Symbol };

fn create_crowdfund_registry_contract(e: &Env) -> (Address, CrowdfundRegistryContract) {
    let contract_id = register_crowdfund_registry(e);
    let crowdfund_registry = CrowdfundRegistryContract::new(e, contract_id.clone());
    (contract_id, crowdfund_registry)
}

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

fn generate_batch_withdraw(env: &Env) -> Vec<Withdraw> {
    vec![env,
        Withdraw {
            drawer: Address::generate(env),
            crowdfund_id: 1,
            
        },
        Withdraw {
            drawer: Address::generate(env),
            crowdfund_id: 2,
        },
        Withdraw {
            drawer: Address::generate(env),
            crowdfund_id: 3,
        },
    ]
}

fn generate_batch_pledge(env: &Env) -> Vec<Pledge> {
    vec![env,
        Pledge {
            donor: Address::generate(env),
            crowdfund_id: 1,
            amount: 50,
        },
        Pledge {
            donor: Address::generate(env),
            crowdfund_id: 2,
            amount: 100,
        },
        Pledge {
            donor: Address::generate(env),
            crowdfund_id: 3,
            amount: 200,
        },
    ]
}

fn generate_crowdfund_args_list(env: &Env, token_address: &Address) -> Vec<CrowdfundArgs> {
    vec![env,
        CrowdfundArgs {
            recipient: Address::generate(env),
            deadline: env.ledger().timestamp() + 10,
            target_amount: 20,
            token: token_address.clone(),
        },
        CrowdfundArgs {
            recipient: Address::generate(env),
            deadline: env.ledger().timestamp() + 10,
            target_amount: 10000,
            token: token_address.clone(),
        },
        CrowdfundArgs {
            recipient: Address::generate(env),
            deadline: env.ledger().timestamp() + 10,
            target_amount: 15000,
            token: token_address.clone(),
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

struct Setup<'a> {
    env: Env,
    registry_admin: Address,
    crowdfund_registry: CrowdfundRegistryContract,
    crowdfund_registry_id: Address,
    token: token::Client<'a>,
    crowdfund_args_list: Vec<CrowdfundArgs>,
    batch_pledge: Vec<Pledge>,
    batch_withdraw: Vec<Withdraw>,
}

impl Setup<'_> {
    fn new() -> Self {
        let env = Env::default();
        let registry_admin = Address::generate(&env);
        let (crowdfund_registry_id, crowdfund_registry) = 
            create_crowdfund_registry_contract(&env);
                // Create the token contract
        let token_admin = Address::generate(&env);
        let (token, token_admin) = create_token_contract(&env, &token_admin);
        let crowdfund_args_list = generate_crowdfund_args_list(&env, &token.address);
        let batch_pledge = generate_batch_pledge(&env);
        let batch_withdraw = generate_batch_withdraw(&env);

        // Mint some tokens to work with
        for pledge in batch_pledge.iter() {
            token_admin.mock_all_auths().mint(&pledge.donor, &1000);
        }

        Self {
            env,
            registry_admin,
            crowdfund_registry,
            crowdfund_registry_id,
            token,
            crowdfund_args_list,
            batch_pledge,
            batch_withdraw,
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
    let counter= setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&setup.crowdfund_args_list);

    assert_eq!(counter, get_counter(env, &setup.crowdfund_registry_id));

}

#[test]
fn test_deposit_to_batch_crowdfunds() {
    let setup = Setup::new();
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&setup.crowdfund_args_list);

    setup
        .crowdfund_registry
        .client()
        .mock_all_auths()
        .deposit_to_batch_crowdfunds(&setup.batch_pledge);

}

#[test]
#[should_panic(expected = "sale is still running")]
fn test_withdraw_from_batch_crowdfunds() {
    let setup = Setup::new();
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&setup.crowdfund_args_list);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .deposit_to_batch_crowdfunds(&setup.batch_pledge);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .withdraw_from_batch_crowdfunds(&setup.batch_withdraw);

}



