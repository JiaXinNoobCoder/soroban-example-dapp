use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Counter,
    Admin,
    CrowdfundMap,
    CrowdfundId(u64),
}
