#![no_std]
use soroban_sdk::token::TokenClient;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, vec, Address, BytesN,
    Env, IntoVal, Symbol, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SettlementError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NodeNotFound = 4,
    ReportNotVerified = 5,
    NoFunds = 6,
    Reentrancy = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeInfo {
    pub uptime_threshold: u64,
    pub escrow_amount: i128,
    pub last_report_time: u64,
    pub cumulative_uptime: u64,
    pub total_paid: i128,
}

#[contracttype]
pub enum DataKey {
    Admin,
    OracleContract,
    Token,
    InProgress,
    Node(Address),
}

#[contract]
pub struct SettlementContract;

#[contractimpl]
impl SettlementContract {
    pub fn initialize(env: Env, admin: Address, oracle_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, SettlementError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::OracleContract, &oracle_contract);
        env.storage().instance().set(&DataKey::InProgress, &false);
    }

    pub fn set_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored {
            panic_with_error!(&env, SettlementError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Token, &token);
    }

    pub fn register_node(
        env: Env,
        admin: Address,
        node: Address,
        uptime_threshold: u64,
        escrow_amount: i128,
    ) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored {
            panic_with_error!(&env, SettlementError::Unauthorized);
        }
        let info = NodeInfo {
            uptime_threshold,
            escrow_amount,
            last_report_time: 0,
            cumulative_uptime: 0,
            total_paid: 0,
        };
        env.storage().instance().set(&DataKey::Node(node), &info);
    }

    pub fn deposit(env: Env, from: Address, amount: i128) {
        from.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&from, env.current_contract_address(), &amount);
    }

    pub fn submit_oracle_report(
        env: Env,
        oracle: Address,
        node: Address,
        uptime: u64,
        timestamp: u64,
        signature: BytesN<64>,
    ) {
        oracle.require_auth();

        let in_progress: bool = env
            .storage()
            .instance()
            .get(&DataKey::InProgress)
            .unwrap_or(false);
        if in_progress {
            panic_with_error!(&env, SettlementError::Reentrancy);
        }
        env.storage().instance().set(&DataKey::InProgress, &true);

        let oracle_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleContract)
            .unwrap();

        let args: Vec<Val> = vec![
            &env,
            node.clone().into_val(&env),
            uptime.into_val(&env),
            timestamp.into_val(&env),
            signature.into_val(&env),
        ];
        let verified: bool =
            env.invoke_contract(&oracle_contract, &Symbol::new(&env, "verify"), args);

        if !verified {
            env.storage().instance().set(&DataKey::InProgress, &false);
            panic_with_error!(&env, SettlementError::ReportNotVerified);
        }

        let mut node_info: NodeInfo = env
            .storage()
            .instance()
            .get(&DataKey::Node(node.clone()))
            .unwrap();

        if uptime >= node_info.uptime_threshold {
            let payout = node_info.escrow_amount * uptime as i128 / 10000;
            if payout > 0 {
                let contract_balance = get_balance_internal(&env);
                let actual_payout = if payout > contract_balance {
                    contract_balance
                } else {
                    payout
                };
                if actual_payout > 0 {
                    let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
                    let token_client = TokenClient::new(&env, &token);
                    token_client.transfer(&env.current_contract_address(), &node, &actual_payout);
                    node_info.total_paid += actual_payout;
                }
            }
        }

        node_info.last_report_time = timestamp;
        node_info.cumulative_uptime += uptime;
        env.storage()
            .instance()
            .set(&DataKey::Node(node), &node_info);
        env.storage().instance().set(&DataKey::InProgress, &false);
    }

    pub fn get_node_info(env: Env, node: Address) -> NodeInfo {
        let key = DataKey::Node(node);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, SettlementError::NodeNotFound))
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn get_oracle_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::OracleContract)
            .unwrap()
    }

    pub fn get_balance(env: Env) -> i128 {
        get_balance_internal(&env)
    }
}

fn get_balance_internal(env: &Env) -> i128 {
    let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
    let token_client = TokenClient::new(env, &token);
    token_client.balance(&env.current_contract_address())
}

mod test;
