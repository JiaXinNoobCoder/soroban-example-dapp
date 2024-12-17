use soroban_sdk::{ Address, BytesN, Env, Symbol, Val, Vec };

// The contract that will be deployed by the deployer contract.
mod contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_crowdfund_contract.wasm"
    );
}

pub fn deploy(
    env: &Env,
    deployer: Address,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
) -> Address {

    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);
    
    let deployed_address = env.deployer().with_address(deployer, salt)
        .deploy(wasm_hash);

    // Invoke the init function with the given arguments.
    let _: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
    // Return the contract ID of the deployed contract and the result of
    // invoking the init result.
    deployed_address
}
