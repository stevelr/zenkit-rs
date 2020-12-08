//! Rust client api for Zenkit
#![deny(missing_docs)]
mod list;
pub use list::{
    fset_f, fset_i, fset_id, fset_s, fset_t, fset_vid, fset_vs, fup_f, fup_i, fup_id, fup_s, fup_t,
    fup_vid, fup_vs, FieldSetVal, FieldVal, ListInfo,
};
mod item;
pub use item::Item;
mod apiclient;
pub use apiclient::{ApiClient, ApiConfig};
mod error;
pub use error::{Error, Result};
mod errorcode;
pub use errorcode::lookup_error;
pub mod types;
mod user;
pub(crate) use user::UserCache;

mod util;
pub use util::log_object;
pub(crate) use util::{f32_or_str, join};

use once_cell::sync::OnceCell;
static API: OnceCell<ApiClient> = OnceCell::new();

/// First-time initialization of Zenkit api client.
/// If api was already initialized, returns Error::AlreadyInitialized
pub fn init_api(config: ApiConfig) -> Result<&'static ApiClient, Error> {
    let api = ApiClient::new(config)?;
    API.set(api).map_err(|_| Error::AlreadyInitialized)?;
    get_api()
}

/// Returns API handle, or error if not initialized
pub fn get_api() -> Result<&'static ApiClient, Error> {
    Ok(API.get().ok_or(Error::NotInitialized)?)
}
