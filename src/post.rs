use crate::media::Media;
use serde::{Deserialize, Serialize};

/// Post contents, metadata, media and interactions
#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub date: u64,
    pub body: Option<String>,
    pub media: Option<Vec<Media>>,
    pub likes: u64,
    pub reposts: u64,
    pub parents: Option<Vec<String>>,
    pub replies: Option<Vec<String>>,
}
