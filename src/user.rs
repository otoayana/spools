use serde::{Deserialize, Serialize};

/// User information and statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: Option<String>,
    pub pfp: Option<String>,
    pub verified: bool,
    pub bio: Option<String>,
    pub followers: u64,
    pub links: Option<Vec<String>>,
    pub posts: Option<Vec<String>>,
}
