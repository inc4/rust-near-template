use std::cmp::Ordering;

use near_sdk::{env, require, StorageUsage};

#[derive(Default)]
pub(crate) struct StorageUsageTrackerData {
    is_tracked: bool,
    pub(crate) last_storage_usage: StorageUsage,
}

impl StorageUsageTrackerData {
    /// Starts tracking storage usage changes
    pub(crate) fn track(mut self) -> Self {
        self.assert_storage_tracking_disabled();
        self.is_tracked = true;
        self.last_storage_usage = env::storage_usage();
        self
    }

    /// Finalizes storage usage changes and returns adjusted value
    pub(crate) fn finish(mut self, storage_usage: StorageUsage) -> StorageUsage {
        self.assert_storage_tracking_enabled();
        self.is_tracked = false;
        let current_storage_usage = env::storage_usage();

        match current_storage_usage.cmp(&self.last_storage_usage) {
            Ordering::Equal => Some(storage_usage),
            Ordering::Greater => {
                storage_usage.checked_add(current_storage_usage - self.last_storage_usage)
            }
            Ordering::Less => {
                storage_usage.checked_sub(self.last_storage_usage - current_storage_usage)
            }
        }
        .unwrap_or_else(|| env::panic_str("Storage computation overflow"))
    }

    fn assert_storage_tracking_disabled(&self) {
        require!(!self.is_tracked, "Storage tracking is already enabled")
    }

    fn assert_storage_tracking_enabled(&self) {
        require!(self.is_tracked, "Storage tracking is not enabled")
    }
}
