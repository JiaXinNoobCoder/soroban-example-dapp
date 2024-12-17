use soroban_sdk::{ Address, Env, Symbol };

pub(crate) fn creat_crowdfund_event(e: &Env, id: u64, recipient: Address, deadline: u64, 
    target_amount: i128, token: Address) {
    let topics = (Symbol::new(e, "creat_crowdfund_event"),);
    let event_payload = (id, recipient, deadline, target_amount, token);
    e.events().publish(topics, event_payload);
}

pub(crate) fn register_init_event(e: &Env, admin: Address) {
    let topics = (Symbol::new(e, "registry_init_success"),);
    let event_payload = admin;
    e.events().publish(topics, event_payload);
}
