use crate::account::Account;
use crate::{Contract, ContractExt};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, near_bindgen, AccountId, Promise};

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.assert_contract_running();

        let deposit_amount = env::attached_deposit();
        if deposit_amount == 0 {
            env::panic_str("No deposit provided")
        }

        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        let registration_only = registration_only.unwrap_or(false);

        match self.get_account_mut(&account_id) {
            // if exists and registration only flag is true, then return deposit to user
            Ok(account) if registration_only => {
                Promise::new(env::predecessor_account_id()).transfer(deposit_amount);
                account.storage_balance()
            }

            // if exists then update near_balance
            Ok(account) => {
                account.storage_balance = account
                    .storage_balance
                    .checked_add(deposit_amount)
                    .unwrap_or_else(|| env::panic_str("Storage balance overflow"));
                account.storage_balance()
            }

            // if not exist and registration only then register and refund
            Err(_) if registration_only => {
                let min_balance = Account::required_deposit(Some(&account_id)).into();

                let refund = deposit_amount.checked_sub(min_balance).unwrap_or_else(|| {
                    env::panic_str("Not enough minimum deposit to register account")
                });

                if refund > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(refund);
                }

                let account = Account::new(&account_id, Some(min_balance));
                let storage_balance = account.storage_balance();

                self.accounts.insert(account_id.clone(), account.into());

                storage_balance
            }

            // else register account with all deposit
            _ => {
                let account = Account::new(&account_id, Some(deposit_amount));
                let storage_balance = account.storage_balance();

                self.accounts.insert(account_id.clone(), account.into());

                storage_balance
            }
        }
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();

        self.assert_contract_running();

        let account_id = env::predecessor_account_id();
        let account = self
            .get_account_mut(&account_id)
            .unwrap_or_else(|e| env::panic_str(e));

        // If amount not provided, use all available storage balance
        let withdraw_amount = amount
            .unwrap_or_else(|| account.storage_balance().available)
            .into();

        account.storage_balance = account
            .storage_balance
            .checked_sub(withdraw_amount)
            .unwrap_or_else(|| env::panic_str("Not enough available storage to withdraw"));

        let storage_balance = account.storage_balance();

        Promise::new(account_id).transfer(withdraw_amount);

        storage_balance
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();

        self.assert_contract_running();

        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);

        match self.remove_account(&account_id) {
            // If account by provided `account_id` not found
            Err(_) => false,

            // If try to unregister a positive balance account without `force` set to `true`
            Ok(account) if account.storage_balance > 0 && !force => env::panic_str(
                "Unable to unregister a positive balance account without `force` set to `true`",
            ),

            // Unregister account and transfer all funds
            Ok(account) => {
                Promise::new(account_id).transfer(account.storage_balance);
                true
            }
        }
    }

    /// Returns storage min/max bounds in $NEAR for account with maximum id length
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: Account::required_deposit(None),
            max: None,
        }
    }

    /// Returns storage balance by `account_id` if account is registered, otherwise None
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.assert_contract_running();
        self.get_account(&account_id)
            .map(|account| account.storage_balance())
            .ok()
    }
}
