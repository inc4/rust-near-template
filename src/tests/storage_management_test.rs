use super::super::account::Account;
use super::super::Contract;
use super::common::*;
use crate::storage_tracker::StorageUsageTrackerData;
use near_contract_standards::storage_management::*;
use near_sdk::test_utils::accounts;
use near_sdk::{env, testing_env, AccountId, ONE_NEAR};

#[test]
fn test_single_account_max_id_len_storage_deposit() {
    // AccountId with max length 64 bytes
    let account_id = AccountId::new_unchecked(
        "n.aabbccddeeffgghhiijjkkllmmnnooppqqrrssttuuvvwwxxyyzz0123456789".to_owned(),
    );

    let context = build_default_context(account_id.clone(), Some(ONE_NEAR), None);

    let mut contract = Contract::init(Some(accounts(0)));

    let storage_tracker = StorageUsageTrackerData::default().track();

    testing_env!(context.build());
    contract.storage_deposit(account_id.clone().into(), Some(false));
    contract.accounts.flush(); // flush content before use env::storage_usage()

    let account_storage_used = storage_tracker.finish(0);

    let account = contract.get_account(&account_id).unwrap();

    assert!(contract.accounts.get(&account_id).is_some());
    assert_eq!(account.storage_usage, account_storage_used)
}

#[test]
fn test_account_storage_deposit_registration_and_deposit() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(ONE_NEAR), None);

    let mut contract = Contract::init(Some(accounts(0)));

    let storage_tracker = StorageUsageTrackerData::default().track();

    testing_env!(context.build());
    contract.storage_deposit(account_id.clone().into(), Some(false));
    contract.accounts.flush(); // flush content before use env::storage_usage()

    let account_storage_used = storage_tracker.finish(0);

    let account = contract.get_account(&account_id).unwrap();

    assert!(contract.accounts.get(&account_id).is_some());
    assert_eq!(account.storage_usage, account_storage_used)
}

#[test]
fn test_account_storage_deposit_registration_only() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(ONE_NEAR), None);

    let mut contract = Contract::init(Some(accounts(0)));

    let storage_tracker = StorageUsageTrackerData::default().track();

    testing_env!(context.build());
    contract.storage_deposit(account_id.clone().into(), Some(true));
    contract.accounts.flush(); // flush content before use env::storage_usage()

    let account_storage_used = storage_tracker.finish(0);

    let account = contract.get_account(&account_id).unwrap();

    assert!(contract.accounts.get(&account_id).is_some());
    assert_eq!(account.storage_usage, account_storage_used);
    assert_eq!(
        account.storage_balance,
        account_storage_used as u128 * env::storage_byte_cost()
    )
}

#[test]
fn test_account_storage_deposit_registration_only_register_twice() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(ONE_NEAR), None);

    let mut contract = Contract::init(Some(accounts(0)));

    let storage_tracker = StorageUsageTrackerData::default().track();

    testing_env!(context.build());
    contract.storage_deposit(account_id.clone().into(), Some(true));
    contract.accounts.flush(); // flush content before use env::storage_usage()

    let account_storage_used = storage_tracker.finish(0);

    let account = contract.get_account(&account_id).unwrap();

    assert!(contract.accounts.get(&account_id).is_some());
    assert_eq!(account.storage_usage, account_storage_used);
    assert_eq!(
        account.storage_balance,
        account_storage_used as u128 * env::storage_byte_cost()
    )
}

#[test]
#[should_panic = "Storage balance overflow"]
fn test_account_storage_deposit_overflow() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(ONE_NEAR), None);

    let mut contract = Contract::init(Some(accounts(0)));

    // register account with maximum allowed deposit
    contract.accounts.insert(
        account_id.clone(),
        Account::new(&account_id, Some(u128::MAX)).into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    testing_env!(context.build());
    contract.storage_deposit(account_id.clone().into(), Some(false));
}

#[test]
#[should_panic = "Not enough available storage to withdraw"]
fn test_storage_withdraw_not_enough_storage_panic() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    // register account with minimum required deposit
    contract.accounts.insert(
        account_id.clone(),
        Account::new(
            &account_id,
            Some(Account::required_deposit(Some(&account_id)).0),
        )
        .into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    // try to withdraw more than have
    testing_env!(context.build());
    contract.storage_withdraw(Some(
        (Account::required_deposit(Some(&account_id)).0 + 1).into(),
    ));
}

#[test]
fn test_storage_withdraw() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    contract.accounts.insert(
        account_id.clone(),
        Account::new(&account_id, Some(ONE_NEAR)).into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    let minimum_account_deposit = Account::required_deposit(Some(&account_id)).0;

    testing_env!(context.build());
    let storage_balance =
        contract.storage_withdraw(Some((ONE_NEAR - minimum_account_deposit).into()));
    assert_eq!(storage_balance.total.0, minimum_account_deposit);
    assert_eq!(storage_balance.available.0, 0);
}

#[test]
#[should_panic = "Unable to unregister a positive balance account without `force` set to `true`"]
fn test_storage_unregister_without_force() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    contract.accounts.insert(
        account_id.clone(),
        Account::new(&account_id, Some(ONE_NEAR)).into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    testing_env!(context.build());
    let unregistered = contract.storage_unregister(Some(false));

    assert_eq!(unregistered, true)
}

#[test]
fn test_storage_unregister_zero_balance() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    contract
        .accounts
        .insert(account_id.clone(), Account::new(&account_id, None).into());
    contract.accounts.flush(); // flush content before use env::storage_usage()

    testing_env!(context.build());
    let unregistered = contract.storage_unregister(Some(false));

    assert_eq!(unregistered, true)
}

#[test]
fn test_storage_unregister_with_force() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    contract.accounts.insert(
        account_id.clone(),
        Account::new(&account_id, Some(ONE_NEAR)).into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    testing_env!(context.build());
    let unregistered = contract.storage_unregister(Some(true));

    assert_eq!(unregistered, true)
}

#[test]
fn test_not_exist_account_storage_unregister() {
    let account_id = accounts(1);

    let context = build_default_context(account_id.clone(), Some(1), None);

    let mut contract = Contract::init(Some(accounts(0)));

    testing_env!(context.build());
    let unregistered = contract.storage_unregister(Some(false));

    assert_eq!(unregistered, false)
}

#[test]
fn test_storage_balance_bounds() {
    let account_id = accounts(1);

    let mut context = build_default_context(account_id.clone(), None, None);

    let contract = Contract::init(Some(accounts(0)));

    testing_env!(context.is_view(true).build());
    let balance_bounds = contract.storage_balance_bounds();

    assert_eq!(balance_bounds.min, Account::required_deposit(None));
    assert_eq!(balance_bounds.max, None);
}

#[test]
fn test_storage_balance_of_not_registered_user() {
    let account_id = accounts(1);

    let mut context = build_default_context(account_id.clone(), None, None);

    let contract = Contract::init(Some(accounts(0)));

    testing_env!(context.is_view(true).build());
    assert!(contract.storage_balance_of(account_id).is_none());
}

#[test]
fn test_storage_balance_of() {
    let account_id = accounts(1);

    let mut context = build_default_context(account_id.clone(), None, None);

    let mut contract = Contract::init(Some(accounts(0)));

    contract.accounts.insert(
        account_id.clone(),
        Account::new(&account_id, Some(ONE_NEAR)).into(),
    );
    contract.accounts.flush(); // flush content before use env::storage_usage()

    testing_env!(context.is_view(true).build());
    let storage_balance = contract.storage_balance_of(account_id.clone()).unwrap();

    assert_eq!(storage_balance.total.0, ONE_NEAR);
    assert_eq!(
        storage_balance.available.0,
        ONE_NEAR - Account::required_deposit(Some(&account_id)).0
    );
}
