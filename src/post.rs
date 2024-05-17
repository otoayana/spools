use crate::media::Media;
use serde::{Deserialize, Serialize};

/// Post contents, metadata, media and interactions
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub date: u64,
    pub body: String,
    pub media: Vec<Media>,
    pub likes: u64,
    pub reposts: u64,
    pub parents: Vec<Subpost>,
    pub replies: Vec<Subpost>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Subpost {
    pub code: String,
    pub name: String,
    pub date: u64,
    pub body: String,
    pub media: Vec<Media>,
    pub likes: u64,
    pub reposts: u64,
}
