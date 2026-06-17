#![cfg(test)]
extern crate std;

use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::SettlementContract;
use mock_oracle::MockOracle;

fn setup_test_env() -> (Env, Address, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let node = Address::generate(&env);
    let oracle = Address::generate(&env);
    let user = Address::generate(&env);

    let oracle_id = env.register(MockOracle, ());
    let settlement_id = env.register(SettlementContract, ());

    let oracle_client = mock_oracle::MockOracleClient::new(&env, &oracle_id);
    oracle_client.initialize(&admin);
    oracle_client.authorize_signer(&admin, &node);

    let settlement_client = crate::SettlementContractClient::new(&env, &settlement_id);
    settlement_client.initialize(&admin, &oracle_id);

    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token.address();
    let sac = StellarAssetClient::new(&env, &token_id);
    sac.mint(&user, &100_000_000);
    sac.mint(&admin, &100_000_000);

    settlement_client.set_token(&admin, &token_id);

    (env, settlement_id, token_id, admin, node, oracle, user)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_contract = Address::generate(&env);
    let contract_id = env.register(SettlementContract, ());
    let client = crate::SettlementContractClient::new(&env, &contract_id);

    client.initialize(&admin, &oracle_contract);
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_oracle_contract(), oracle_contract);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_contract = Address::generate(&env);
    let contract_id = env.register(SettlementContract, ());
    let client = crate::SettlementContractClient::new(&env, &contract_id);

    client.initialize(&admin, &oracle_contract);
    client.initialize(&admin, &oracle_contract);
}

#[test]
fn test_register_node() {
    let (env, settlement_id, _token_id, admin, node, _oracle, _user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.register_node(&admin, &node, &9000, &100_000);

    let info = client.get_node_info(&node);
    assert_eq!(info.uptime_threshold, 9000);
    assert_eq!(info.escrow_amount, 100_000);
    assert_eq!(info.last_report_time, 0);
    assert_eq!(info.cumulative_uptime, 0);
    assert_eq!(info.total_paid, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_register_node_unauthorized() {
    let (env, settlement_id, _token_id, _admin, node, _oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.register_node(&user, &node, &9000, &100_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_get_nonexistent_node() {
    let (env, settlement_id, _token_id, _admin, _node, _oracle, _user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    let ghost = Address::generate(&env);
    client.get_node_info(&ghost);
}

#[test]
fn test_deposit() {
    let (env, settlement_id, token_id, _admin, _node, _oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);
    let token_client = TokenClient::new(&env, &token_id);

    let user_balance_before = token_client.balance(&user);
    let contract_balance_before = token_client.balance(&settlement_id);

    client.deposit(&user, &50_000);

    assert_eq!(token_client.balance(&user), user_balance_before - 50_000);
    assert_eq!(
        token_client.balance(&settlement_id),
        contract_balance_before + 50_000
    );
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_deposit_zero() {
    let (env, settlement_id, _token_id, _admin, _node, _oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.deposit(&user, &0);
}

#[test]
fn test_submit_oracle_report_above_threshold() {
    let (env, settlement_id, token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);
    let token_client = TokenClient::new(&env, &token_id);

    client.deposit(&user, &200_000);
    assert_eq!(client.get_balance(), 200_000);

    client.register_node(&admin, &node, &8000, &50_000);

    let node_balance_before = token_client.balance(&node);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_oracle_report(&oracle, &node, &9500, &1000, &signature);

    let expected_payout: i128 = 50_000 * 9500 / 10000;
    assert_eq!(
        token_client.balance(&node),
        node_balance_before + expected_payout
    );
    assert_eq!(client.get_balance(), 200_000 - expected_payout);

    let info = client.get_node_info(&node);
    assert_eq!(info.last_report_time, 1000);
    assert_eq!(info.cumulative_uptime, 9500);
    assert_eq!(info.total_paid, expected_payout);
}

#[test]
fn test_submit_oracle_report_below_threshold() {
    let (env, settlement_id, _token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.deposit(&user, &200_000);
    client.register_node(&admin, &node, &8000, &50_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_oracle_report(&oracle, &node, &5000, &2000, &signature);

    let info = client.get_node_info(&node);
    assert_eq!(info.last_report_time, 2000);
    assert_eq!(info.total_paid, 0);
    assert_eq!(info.escrow_amount, 50_000);
}

#[test]
fn test_submit_oracle_report_zero_uptime() {
    let (env, settlement_id, _token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.deposit(&user, &200_000);
    client.register_node(&admin, &node, &1000, &50_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_oracle_report(&oracle, &node, &0, &3000, &signature);

    let info = client.get_node_info(&node);
    assert_eq!(info.total_paid, 0);
    assert_eq!(info.cumulative_uptime, 0);
}

#[test]
fn test_multiple_oracle_reports() {
    let (env, settlement_id, token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);
    let _token_client = TokenClient::new(&env, &token_id);

    client.deposit(&user, &500_000);
    client.register_node(&admin, &node, &5000, &100_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);

    client.submit_oracle_report(&oracle, &node, &6000, &1000, &signature);
    let info = client.get_node_info(&node);
    assert_eq!(info.total_paid, 100_000 * 6000 / 10000);

    client.submit_oracle_report(&oracle, &node, &9000, &2000, &signature);
    let info = client.get_node_info(&node);
    let expected_total = (100_000 * 6000 / 10000) + (100_000 * 9000 / 10000);
    assert_eq!(info.total_paid, expected_total);
    assert_eq!(info.cumulative_uptime, 15000);
}

#[test]
fn test_reentrancy_guard_clears_after_successful_report() {
    let (env, settlement_id, _token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.deposit(&user, &100_000);
    client.register_node(&admin, &node, &0, &100_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_oracle_report(&oracle, &node, &10000, &1000, &signature);

    let info = client.get_node_info(&node);
    assert_eq!(info.total_paid, 100_000);
}

#[test]
fn test_balance_multiple_deposits() {
    let (env, settlement_id, _token_id, _admin, _node, _oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);

    client.deposit(&user, &10_000);
    assert_eq!(client.get_balance(), 10_000);

    client.deposit(&user, &20_000);
    assert_eq!(client.get_balance(), 30_000);

    client.deposit(&user, &70_000);
    assert_eq!(client.get_balance(), 100_000);
}

#[test]
fn test_full_payout_cycle() {
    let (env, settlement_id, token_id, admin, node, oracle, user) = setup_test_env();
    let client = crate::SettlementContractClient::new(&env, &settlement_id);
    let token_client = TokenClient::new(&env, &token_id);

    let contract_before = client.get_balance();
    assert_eq!(contract_before, 0);

    client.deposit(&user, &100_000);
    assert_eq!(client.get_balance(), 100_000);

    client.register_node(&admin, &node, &0, &100_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_oracle_report(&oracle, &node, &10000, &5000, &signature);

    let info = client.get_node_info(&node);
    assert_eq!(info.total_paid, 100_000);
    assert_eq!(token_client.balance(&node), 100_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_unverified_oracle_report() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let node = Address::generate(&env);
    let unauthorized_oracle = Address::generate(&env);
    let user = Address::generate(&env);

    let oracle_id = env.register(MockOracle, ());
    let settlement_id = env.register(SettlementContract, ());

    let oracle_client = mock_oracle::MockOracleClient::new(&env, &oracle_id);
    oracle_client.initialize(&admin);

    let settlement_client = crate::SettlementContractClient::new(&env, &settlement_id);
    settlement_client.initialize(&admin, &oracle_id);

    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token.address();
    let sac = StellarAssetClient::new(&env, &token_id);
    sac.mint(&user, &100_000);
    settlement_client.set_token(&admin, &token_id);
    settlement_client.deposit(&user, &100_000);
    settlement_client.register_node(&admin, &node, &5000, &50_000);

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    settlement_client.submit_oracle_report(&unauthorized_oracle, &node, &9500, &1000, &signature);
}
