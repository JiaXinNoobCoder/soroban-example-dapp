#![cfg(test)]
// The contract that will be deployed by the deployer contract.
mod crowdfund_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_crowdfund_contract.wasm"
    );
}

extern crate std;
use super::testutils::{register_test_contract as register_crowdfund_registry, capture_stdout, CrowdfundRegistryContract};
use super::storage_types::DataKey;
use super::entity::*;
use soroban_sdk::testutils::Events;
use soroban_sdk::{ token, IntoVal, vec, Env, Map, testutils::{ Address as TestAddress, Ledger }, Address, Vec, Val, Symbol };

fn advance_ledger(e: &Env, delta: u64) {
    e.ledger().with_mut(|l| {
        l.timestamp += delta;
    });
}

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

fn generate_batch_withdraw(env: &Env, drawer: &Vec<Address>) -> Vec<Withdraw> {
    vec![env,
        Withdraw {
            drawer: drawer.get(0).unwrap(),
            crowdfund_id: 1,
            
        },
        Withdraw {
            drawer: drawer.get(1).unwrap(),
            crowdfund_id: 2,
        },
        Withdraw {
            drawer: drawer.get(2).unwrap(),
            crowdfund_id: 3,
        },
    ]
}

fn generate_batch_pledge(env: &Env, donors: &Vec<Address>, 
    donations: &Vec<i128>) -> Vec<Pledge> {
    vec![env,
        Pledge {
            donor: donors.get(0).unwrap().clone(),
            crowdfund_id: 1,
            amount: donations.get(0).unwrap(),
        },
        Pledge {
            donor: donors.get(1).unwrap().clone(),
            crowdfund_id: 2,
            amount: donations.get(1).unwrap(),
        },
        Pledge {
            donor: donors.get(2).unwrap().clone(),
            crowdfund_id: 3,
            amount: donations.get(2).unwrap(),
        },
    ]
}

fn generate_crowdfund_args_list(env: &Env, recipients: &Vec<Address>, targets: &Vec<i128>, 
    token_address: &Address) -> Vec<CrowdfundArgs> {

    vec![env,
        CrowdfundArgs {
            recipient: recipients.get(0).unwrap().clone(),
            deadline: env.ledger().timestamp() + 10,
            target_amount: targets.get(0).unwrap(),
            token: token_address.clone(),
        },
        CrowdfundArgs {
            recipient: recipients.get(1).unwrap().clone(),
            deadline: env.ledger().timestamp() + 10,
            target_amount: targets.get(1).unwrap(),
            token: token_address.clone(),
        },
        CrowdfundArgs {
            recipient: recipients.get(2).unwrap().clone(),
            deadline: env.ledger().timestamp() + 10,
            target_amount: targets.get(2).unwrap(),
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
    donors: Vec<Address>,
    recipients: Vec<Address>,
    crowdfund_registry: CrowdfundRegistryContract,
    crowdfund_registry_id: Address,
    token: token::Client<'a>,
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
        let donors = vec![&env, Address::generate(&env), Address::generate(&env), Address::generate(&env)];
        let recipients = vec![&env, Address::generate(&env), Address::generate(&env), Address::generate(&env)];


        // Mint some tokens to work with
        for donor in donors.iter() {
            token_admin.mock_all_auths().mint(&donor, &1000);
        }

        Self {
            env,
            registry_admin,
            donors,
            recipients,
            crowdfund_registry,
            crowdfund_registry_id,
            token,
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
fn test_creat_batch_crowdfunds_v1() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    let targets = vec![&env, 100, 200, 300];

    let crowdfund_args_list = generate_crowdfund_args_list(env, &setup.recipients, 
        &targets, &setup.token.address);
    let ids= setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&crowdfund_args_list);
    capture_stdout(|| {
        env.budget().print();
    }, "creat_batch_crowdfunds.txt").expect("Failed to capture stdout");
    assert_eq!(vec![env, 1, 2, 3], ids);

}

#[test]
fn test_creat_batch_crowdfunds_v2() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    let targets = vec![&env, 100, 200, 300];

    let crowdfund_args_list = generate_crowdfund_args_list(env, &setup.recipients, 
        &targets, &setup.token.address);

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(crowdfund_contract::WASM);

    let ids= setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds_v2(&wasm_hash, &crowdfund_args_list);
    
    capture_stdout(|| {
        env.budget().print();
    }, "creat_batch_crowdfunds_v2.txt").expect("Failed to capture stdout");    

    assert_eq!(vec![env, 1, 2, 3], ids);

}

#[test]
fn test_deposit_to_batch_crowdfunds() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    let targets = vec![&env, 100, 200, 300];
    let crowdfund_args_list = generate_crowdfund_args_list(env, &setup.recipients, 
            &targets,&setup.token.address);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&crowdfund_args_list);
    let donations = vec![&env, 100, 200, 300];
    let batch_pledge = generate_batch_pledge(env, &setup.donors, &donations); 

    setup
        .crowdfund_registry
        .client()
        .mock_all_auths()
        .deposit_to_batch_crowdfunds(&batch_pledge);

    let crowdfund_map= get_map(env, &setup.crowdfund_registry_id);
    let crowdfund1 = crowdfund_map.get(1).unwrap();
    let crowdfund2 = crowdfund_map.get(2).unwrap();
    let crowdfund3 = crowdfund_map.get(3).unwrap();

    assert_eq!(batch_pledge.get(0).unwrap().amount, 
    setup.token.balance(&crowdfund1));

    assert_eq!(batch_pledge.get(1).unwrap().amount, 
    setup.token.balance(&crowdfund2));

    assert_eq!(batch_pledge.get(2).unwrap().amount, 
    setup.token.balance(&crowdfund3));

    assert_eq!(900, 
    setup.token.balance(&setup.donors.get(0).unwrap()));

    assert_eq!(800, 
    setup.token.balance(&setup.donors.get(1).unwrap()));

    assert_eq!(700, 
    setup.token.balance(&setup.donors.get(2).unwrap()));

}

#[test]
fn test_withdraw_from_batch_crowdfunds_target_hitted() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);
    let targets = vec![&env, 100, 200, 300];
    let crowdfund_args_list = generate_crowdfund_args_list(env, &setup.recipients, 
            &targets,&setup.token.address);

    
    let donations = vec![&env, 100, 200, 300];
    let batch_pledge = generate_batch_pledge(env, &setup.donors, &donations); 

    let batch_withdraw = generate_batch_withdraw(env, &setup.recipients);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&crowdfund_args_list);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .deposit_to_batch_crowdfunds(&batch_pledge);

    advance_ledger(env, 15);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .withdraw_from_batch_crowdfunds(&batch_withdraw);

    assert_eq!(100, setup.token.balance(&setup.recipients.get(0).unwrap()));
    assert_eq!(200, setup.token.balance(&setup.recipients.get(1).unwrap()));
    assert_eq!(300, setup.token.balance(&setup.recipients.get(2).unwrap()));

}


#[test]
fn test_withdraw_from_batch_crowdfunds_expire() {
    let setup = Setup::new();
    let env = &setup.env;
    setup
        .crowdfund_registry
        .client()
        .initialize(&setup.registry_admin);

    let targets = vec![&env, 100, 300, 300];
    let crowdfund_args_list = generate_crowdfund_args_list(env, &setup.recipients, 
            &targets,&setup.token.address);
    
    let donations = vec![&env, 50, 50, 50];
    let batch_pledge = generate_batch_pledge(env, &setup.donors, &donations); 
    let batch_withdraw = generate_batch_withdraw(env, &setup.donors);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .creat_batch_crowdfunds(&crowdfund_args_list);

    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .deposit_to_batch_crowdfunds(&batch_pledge);

    assert_eq!(950, setup.token.balance(&setup.donors.get(0).unwrap()));
    assert_eq!(950, setup.token.balance(&setup.donors.get(1).unwrap()));
    assert_eq!(950, setup.token.balance(&setup.donors.get(2).unwrap()));

    advance_ledger(env, 15);
    
    setup
      .crowdfund_registry
      .client()
      .mock_all_auths()
      .withdraw_from_batch_crowdfunds(&batch_withdraw);

    assert_eq!(1000, setup.token.balance(&setup.donors.get(0).unwrap()));
    assert_eq!(1000, setup.token.balance(&setup.donors.get(1).unwrap()));
    assert_eq!(1000, setup.token.balance(&setup.donors.get(2).unwrap()));

}


