use soroban_sdk::{ symbol_short, IntoVal, Address, BytesN, Env, Symbol, Val, Vec };

// The contract that will be deployed by the deployer contract.
mod contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_crowdfund_contract.wasm"
    );
}

const DEPOSIT: Symbol = symbol_short!("deposit");
const WITHDRAW: Symbol = symbol_short!("withdraw");

pub fn deposit(env: &Env, crowdfund_address: Address, donor: Address, amount: i128) {
    // Skip authorization if donor is the current contract.
    if donor != env.current_contract_address() {
        donor.require_auth();
    }

    let args= (donor, amount).into_val(env);
    // Invoke the deposit function on the crowdfund contract.
    let _: Val = env.invoke_contract(&crowdfund_address, &DEPOSIT, args);

}

pub fn withdraw(env: &Env, crowdfund_address: Address, drawer: Address) {
    // Skip authorization if donor is the current contract.
    if drawer != env.current_contract_address() {
        drawer.require_auth();
    }
    let args: Vec<Val>= (drawer,).into_val(env);

    // Invoke the deposit function on the crowdfund contract.
    let _: Val = env.invoke_contract(&crowdfund_address, &WITHDRAW, args);
}


pub fn deploy(
    env: &Env,
    deployer: Address,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
) -> Address {

    // Skip authorization if deployer is the current contract.
    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);
    
    let deployed_address = env.deployer().with_address(deployer, salt)
        .deploy(wasm_hash);

    // Invoke the init function with the given arguments.
    let _: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
    // Return the contract ID of the deployed contract and the result of
    // invoking the init result.
    deployed_address
}

pub fn deploy_v2(
    env: &Env,
    deployer: Address,
    wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_args: Vec<Val>,
) -> Address {

    // Skip authorization if deployer is the current contract.
    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    
    let deployed_address = env.deployer().with_address(deployer, salt)
        .deploy(wasm_hash);

    // Invoke the init function with the given arguments.
    let _: Val = env.invoke_contract(&deployed_address, &Symbol::new(env, "initialize"), init_args);
    // Return the contract ID of the deployed contract and the result of
    // invoking the init result.
    deployed_address
}