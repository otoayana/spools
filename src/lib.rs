//! # spools
//! spools is a content scraping library for Instagram's [Threads](https://threads.net).
//! spools aims to provide a more comfortable interface than Threads' cumbersome and obfuscated
//! internal API, with the added bonus of not requiring an account.
//!
//! ## Making a client
//! In order to use any of the provided methods, creating a client is required.
//! User and post fetching by ID are provided through [`Threads`] methods.
//!
//! ```rust
//! # use spools;
//! #
//! # async fn run() -> Result<(), spools::SpoolsError> {
//! let client = spools::Threads::new()?;
//! let user = client.fetch_user("zuck").await?;
//! let post = client.fetch_post(&user.posts[0].code).await?;
//! #     Ok(())
//! # }
mod error;
mod media;
mod post;
mod threads;
mod user;

pub use error::SpoolsError;
pub use media::{Media, MediaKind};
pub use post::{Post, Subpost};
pub use threads::Threads;
pub use user::{Author, User};

#[cfg(test)]
mod test;
