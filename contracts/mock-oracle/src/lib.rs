#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map};

#[contracttype]
pub enum OracleDataKey {
    Admin,
    AuthorizedSigners,
}

#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&OracleDataKey::Admin, &admin);
        let signers: Map<Address, bool> = Map::new(&env);
        env.storage()
            .instance()
            .set(&OracleDataKey::AuthorizedSigners, &signers);
    }

    pub fn authorize_signer(env: Env, admin: Address, signer: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&OracleDataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let mut signers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&OracleDataKey::AuthorizedSigners)
            .unwrap();
        signers.set(signer, true);
        env.storage()
            .instance()
            .set(&OracleDataKey::AuthorizedSigners, &signers);
    }

    pub fn verify(
        env: Env,
        node: Address,
        uptime: u64,
        _timestamp: u64,
        _signature: BytesN<64>,
    ) -> bool {
        let signers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&OracleDataKey::AuthorizedSigners)
            .unwrap();
        signers.get(node).unwrap_or(false) && uptime <= 10000
    }
}
