use crate::account::{Account, VAccount};
use crate::misc::RunningState;
use crate::storage::StorageKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// The contract's owner account id
    pub(crate) owner_id: AccountId,
    /// Contract's state, e.g. running, paused
    pub(crate) running_state: RunningState,
    /// User versioned accounts data keyed by AccountId
    pub(crate) accounts: LookupMap<AccountId, VAccount>,
}

#[near_bindgen]
impl Contract {
    /// Initializes contract
    #[init]
    pub fn init(owner_id: Option<AccountId>) -> Self {
        Self {
            owner_id: owner_id.unwrap_or_else(env::predecessor_account_id),
            running_state: RunningState::Running,
            accounts: LookupMap::new(StorageKey::Accounts),
        }
    }
}

impl Contract {
    /// Checks if contract is at running state
    pub(crate) fn assert_contract_running(&self) {
        require!(
            self.running_state == RunningState::Running,
            "Contract paused"
        );
    }

    /// Asserts if the caller is not an owner of the contract
    pub(crate) fn assert_owner(&self) {
        require!(self.is_owner(&env::predecessor_account_id()), "Not allowed");
    }

    /// Checks ifn the caller is an owner of the contract
    pub(crate) fn is_owner(&self, account_id: &AccountId) -> bool {
        account_id == &self.owner_id
    }

    /// Returns reference to account by provided `account_id`
    pub(crate) fn remove_account(
        &mut self,
        account_id: &AccountId,
    ) -> Result<Account, &'static str> {
        self.accounts
            .remove(account_id)
            .map(Account::from)
            .ok_or("Account is not registered")
    }

    /// Returns reference to account by provided `account_id`
    pub(crate) fn get_account<'a>(
        &'a self,
        account_id: &'a AccountId,
    ) -> Result<&'a Account, &'static str> {
        self.accounts
            .get(account_id)
            .map(<&'a Account>::from)
            .ok_or("Account is not registered")
    }

    /// Returns mutable reference to account by provided `account_id`
    pub(crate) fn get_account_mut<'a>(
        &'a mut self,
        account_id: &'a AccountId,
    ) -> Result<&'a mut Account, &'static str> {
        self.accounts
            .get_mut(account_id)
            .map(<&'a mut Account>::from)
            .ok_or("Account is not registered")
    }
}
