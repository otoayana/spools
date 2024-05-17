use crate::{error::SpoolsError, post::Subpost, Threads};
use serde::{Deserialize, Serialize};

/// User information and statistics
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub pfp: String,
    pub verified: bool,
    pub bio: String,
    pub followers: u64,
    pub links: Vec<String>,
    pub posts: Vec<Subpost>,
}

/// User embedded within object
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    pub username: String,
    pub pfp: String,
    pub verified: bool,
}

impl Author {
    /// Convert author into its detailed counterpart
    pub async fn to_user(&self) -> Result<User, SpoolsError> {
        let client = Threads::new()?;
        let user = client.fetch_user(&self.username).await?;

        Ok(user)
    }
}
