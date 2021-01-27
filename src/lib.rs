//! Rust client api for Zenkit
#![deny(missing_docs)]
mod apiclient;
pub use apiclient::{ApiClient, ApiConfig};
mod error;
pub use error::Error;
mod errorcode;
pub use errorcode::lookup_error;
mod item;
mod list;
pub mod types;
mod user;
pub(crate) use user::UserCache;
pub(crate) mod datetime;
mod util;
pub(crate) use util::{f32_or_str, join};

use once_cell::sync::OnceCell;
static API: OnceCell<ApiClient> = OnceCell::new();

/// First-time initialization of Zenkit api client.
/// If api was already initialized, returns Error::AlreadyInitialized
/// ```rust
/// use zenkit::{init_api,ApiConfig};
/// let api = init_api(ApiConfig::default()).unwrap();
/// ```
pub fn init_api(config: ApiConfig) -> Result<&'static ApiClient, Error> {
    let api = ApiClient::new(config)?;
    API.set(api).map_err(|_| Error::AlreadyInitialized)?;
    get_api()
}

/// Returns API handle, or error if not initialized
pub fn get_api() -> Result<&'static ApiClient, Error> {
    Ok(API.get().ok_or(Error::NotInitialized)?)
}

/// Zenkit API common Traits and structs
pub mod prelude {
    pub use crate::types::{DateTime, Utc, ZKObjectID};
}
