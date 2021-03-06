use crate::types::ErrorInfo;
use std::fmt;

/// Errors returned by this crate
#[derive(Debug)]
pub enum Error {
    /// Error returned from Zenkit: (http status, optional zenkit-defined error code)
    ApiError(u16, Option<ErrorInfo>),
    /// Error serializing or deserializing json data
    JsonError(String),
    /// Network error (reported by reqwest http client)
    //NetError(String),
    /// Invalid utf-8 (converting binary to text)
    Utf8Error(String),
    /// URL parsing error
    ParseError(String),
    /// IO error - used for verbose http logging to files
    IoError(String),
    /// Error returned by reqwest library
    Reqwest(reqwest::Error),

    /// assumed single-value category field but multiple values were set
    /// First param is general message, second is field name
    MultiCategory(String, String),

    /// Api token has not been set
    MissingApiToken(String),

    /// Error if static object is already initialized
    AlreadyInitialized,
    /// Error if static initializer was not called before getter
    NotInitialized,

    /// Error that doesn't fit any of the above
    Other(String),
}

impl Error {
    /// Returns true if the error means a rate limit has been hit
    pub fn is_rate_limit(&self) -> bool {
        if let Error::ApiError(_, Some(info)) = self {
            if &info.code == "D1" || &info.code == "D2" {
                return true;
            }
        }
        false
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonError(format!("serde_json: {:?}", e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(format!("{:?}", e))
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8Error(format!("Invalid UTF-8: {}", e.to_string()))
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        Error::Other(format!(
            "Thread died while holding lock. Sorry, you need to quit and start over: {}",
            e
        ))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
