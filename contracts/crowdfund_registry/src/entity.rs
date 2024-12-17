use soroban_sdk::{ contracttype, Address };

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrowdfundArgs {
    pub creator: Address,
    pub recipient: Address,
    pub deadline: u64,
    pub target_amount: i128,
    pub token: Address
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pledge {
    pub donor: Address,
    pub amount: i128,
    pub crowdfund_id: u64
}