use std::str::FromStr;

use super::utils::{build_contract, gen_user_account};
use near_contract_standards::storage_management::StorageBalance;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{ONE_NEAR, ONE_YOCTO};
use workspaces::{Account, Contract};

#[tokio::test]
async fn test_user_registration_and_withdraw() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = build_contract(&worker, "./").await?;

    // generate sub-account and transfer funds
    let user1 = gen_user_account(&worker, "user1.test.near").await?;
    let user1_account_id = near_sdk::AccountId::from_str(user1.id().as_str())?;

    let deposit = 3 * ONE_NEAR;
    let withdraw = ONE_NEAR;
    let storage_balance_left = deposit - withdraw;

    let user1_storage_balance = storage_deposit(&contract, &user1, deposit, None).await?;
    assert_eq!(user1_storage_balance.total.0, deposit);

    let user1_storage_balance = storage_withdraw(&contract, &user1, Some(withdraw.into())).await?;
    assert_eq!(user1_storage_balance.total.0, storage_balance_left);

    let opt_user1_storage_balance = storage_balance_of(&contract, &user1).await?;
    match opt_user1_storage_balance {
        Some(StorageBalance { total, available })
            if total.0 == storage_balance_left
                && available.0
                    == storage_balance_left
                        - crate::account::Account::required_deposit(Some(&user1_account_id)).0 => {}
        _ => panic!("Verify user `{:?}` storage balance failed!", user1.id()),
    };

    Ok(())
}

#[tokio::test]
async fn test_user_registration_only() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = build_contract(&worker, "./").await?;

    // generate sub-account and transfer funds
    let user1 = gen_user_account(&worker, "user1.test.near").await?;
    let user1_account_id = near_sdk::AccountId::from_str(user1.id().as_str())?;

    // register user at OCC contract
    storage_deposit(&contract, &user1, ONE_NEAR, Some(true)).await?;

    let user1_storage_balance = storage_balance_of(&contract, &user1).await?;
    match user1_storage_balance {
        Some(StorageBalance { total, available })
            if total.0 == crate::account::Account::required_deposit(Some(&user1_account_id)).0
                && available.0 == 0 => {}
        _ => panic!("Verify user `{:?}` storage balance failed!", user1.id()),
    };

    Ok(())
}

async fn storage_deposit(
    contract: &Contract,
    target_account: &Account,
    deposit: u128,
    registration_only: Option<bool>,
) -> anyhow::Result<StorageBalance> {
    let res = target_account
        .call(contract.id(), "storage_deposit")
        .args_json(json!({
            "registration_only": registration_only,
        }))
        .deposit(deposit)
        .max_gas()
        .transact()
        .await?;

    match res.clone().into_result() {
        Ok(res) => res
            .json::<StorageBalance>()
            .map_err(|e| anyhow::Error::msg(format!("Parse `StorageBalance` failed. {:?}", e))),
        Err(_) => Err(anyhow::Error::msg(format!(
            "Registration of account `{:?}` failed on contract `{:?}`. Log {:?}",
            target_account.id(),
            contract.id(),
            res
        ))),
    }
}

async fn storage_withdraw(
    contract: &Contract,
    target_account: &Account,
    amount: Option<U128>,
) -> anyhow::Result<StorageBalance> {
    let res = target_account
        .call(contract.id(), "storage_withdraw")
        .args_json(json!({
            "amount": amount,
        }))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?;

    match res.clone().into_result() {
        Ok(res) => res
            .json::<StorageBalance>()
            .map_err(|e| anyhow::Error::msg(format!("Parse `StorageBalance` failed. {:?}", e))),
        Err(_) => Err(anyhow::Error::msg(format!(
            "Registration of account `{:?}` failed on contract `{:?}`. Log {:?}",
            target_account.id(),
            contract.id(),
            res
        ))),
    }
}

async fn storage_balance_of(
    contract: &Contract,
    account: &Account,
) -> anyhow::Result<Option<StorageBalance>> {
    let res = contract
        .view("storage_balance_of")
        .args_json(json!({
            "account_id": account.id(),
        }))
        .await;

    match res {
        Ok(res) => res
            .json::<Option<StorageBalance>>()
            .map_err(|e| anyhow::Error::msg(format!("Parse `StorageBalance` failed. {:?}", e))),
        Err(_) => Err(anyhow::Error::msg(format!(
            "Get storage balance of account `{:?}` failed. Log {:?}",
            account.id(),
            res
        ))),
    }
}
