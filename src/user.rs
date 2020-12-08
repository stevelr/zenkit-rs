use crate::types::User;
use std::{clone::Clone, sync::Arc};

/// Cached array of users
#[derive(Debug, Default)]
pub(crate) struct UserCache {
    users: Vec<Arc<User>>,
}

impl UserCache {
    pub fn replace_all(&mut self, users: Vec<Arc<User>>) {
        self.users = users
    }

    /// Returns true if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// Returns list of workspace users.
    /// For searching for a user based on a boolean predicate,
    /// find_user is more efficient, because it doesn't allocate a new vector.
    pub fn users(&self) -> Vec<Arc<User>> {
        self.users.clone()
    }

    /// Find first user matching predicate
    pub fn find_user<P>(&self, predicate: P) -> Option<Arc<User>>
    where
        P: Fn(&Arc<User>) -> bool,
    {
        for u in self.users.iter() {
            if (predicate)(u) {
                return Some(u.clone());
            }
        }
        None
    }

    /*
    /// Finds user_id. Parameter may be display name, full name, uuid, or id
    /// Name search is case-insensitive.
    /// Reurns None if not found
    pub async fn get_user_id(&'_ self, user: &str) -> Option<ID> {
        if let Ok(id) = user.parse::<u64>() {
            return Some(id);
        }
        let lc_name = user.to_lowercase();
        self.find_user(|u| {
            u.display_name.to_lowercase() == lc_name
                || u.full_name.to_lowercase() == lc_name
                || u.uuid == lc_name
        })
        .map(|u| u.id)
    }
    */
}
