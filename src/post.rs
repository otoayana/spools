use crate::{media::Media, Threads};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Post contents, metadata, media and interactions
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

/// Posts embedded within other objects
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Subpost {
    pub code: String,
    pub name: String,
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

impl Post {
    pub async fn to_subpost(&self, code: Option<String>) -> Result<Subpost> {
        let final_code: String;

        if let Some(val) = code {
            final_code = val;
        } else {
            let client = Threads::new()?;
            final_code = client.fetch_post_code(&self.id).await?;
        }

        Ok(Subpost {
            code: final_code,
            name: self.name.to_owned(),
            date: self.date,
            body: self.body.to_owned(),
            media: self.media.to_owned(),
            likes: self.likes,
            reposts: self.reposts,
        })
    }
}
