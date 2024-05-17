use crate::{media::Media, user::Author, Threads};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Post contents, metadata, media and interactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: String,
    pub author: Author,
    pub date: u64,
    pub body: String,
    pub media: Vec<Media>,
    pub likes: u64,
    pub reposts: u64,
    pub parents: Vec<Subpost>,
    pub replies: Vec<Subpost>,
}

/// Posts embedded within other objects
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subpost {
    pub code: String,
    pub author: Author,
    pub date: u64,
    pub body: String,
    pub media: Vec<Media>,
    pub likes: u64,
    pub reposts: u64,
}

impl Subpost {
    pub async fn to_post(&self) -> Result<Post> {
        let client = Threads::new()?;
        let post = client.fetch_post(&self.code).await?;

        Ok(post)
    }
}
