#![cfg(test)]

extern crate std;
use crate::CrowdfundRegistryContractClient;

use soroban_sdk::{Address, Env};

use std::fs::File;
use std::io::{self};
use std::os::unix::io::AsRawFd;

pub fn capture_stdout<F: FnOnce()>(f: F, file_path: &str) -> io::Result<()> {
    let file = File::create(file_path)?;
    let stdout = io::stdout();
    let stdout_fd = stdout.as_raw_fd();
    let file_fd = file.as_raw_fd();

    unsafe {
        libc::dup2(file_fd, stdout_fd);
    }

    f();

    unsafe {
        libc::dup2(stdout_fd, file_fd);
    }

    Ok(())
}

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
