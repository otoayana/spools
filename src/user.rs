use crate::post::Subpost;
use serde::{Deserialize, Serialize};

/// User information and statistics
#[derive(Debug, Serialize, Deserialize)]
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
