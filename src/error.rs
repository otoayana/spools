use core::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpoolsError {
    #[error("{0} not found")]
    NotFound(Types),
    #[error("endpoint returned invalid response")]
    InvalidResponse,
    #[error("unable to fetch request: {0}")]
    RequestError(reqwest::Error),
    #[error("couldn't build client")]
    ClientError,
    #[error("couldn't build subpost")]
    SubpostError,
}

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
