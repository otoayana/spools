use core::fmt;
use serde_json::Value;
use std::fmt::Write;
use thiserror::Error;

/// Error type for spools
#[derive(Error, Debug)]
pub enum SpoolsError {
    #[error("{0} not found")]
    NotFound(Types),
    #[error("endpoint returned invalid response")]
    InvalidResponse,
    #[error("endpoint returned the following errors: {0}")]
    ResponseError(String),
    #[error("unable to fetch request: {0}")]
    RequestError(reqwest::Error),
    #[error("couldn't build client")]
    ClientError,
    #[error("couldn't build subpost")]
    SubpostError,
}

/// Possible objects to be fetched on a request. Used for errors.
#[derive(Debug)]
pub enum Types {
    Post,
    User,
}

impl fmt::Display for Types {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        let out = match self {
            Types::Post => "post",
            Types::User => "user",
        };

        write!(f, "{}", out)
    }
}

impl SpoolsError {
    pub(crate) fn deserialize_error(response: Value) -> Self {
        let maybe_error = response.pointer("/errors");

        if let Some(Value::Array(error_array)) = maybe_error {
            SpoolsError::ResponseError(error_array.iter().fold(String::new(), |mut out, err| {
                let _ = write!(
                    out,
                    "{};",
                    err.pointer("/summary")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .to_owned()
                        .unwrap_or_default()
                );

                out
            }))
        } else {
            SpoolsError::InvalidResponse
        }
    }
}
