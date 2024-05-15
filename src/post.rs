use crate::{media::Media, threads::Threads};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tokio::task;

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

impl Post {
    /// Fetches the post's parents, in case this is a reply to a post
    pub async fn fetch_parents(&self, limit: Option<Range<usize>>) -> Result<Vec<Post>> {
        let mut posts: Vec<Post> = vec![];

        if let Some(ids) = &self.parents {
            let mut cloned = ids.clone();

            if let Some(num) = limit {
                if num.clone().into_iter().any(|x| x >= cloned.len()) {
                    return Err(Error::msg("range is out of bounds"));
                }

                cloned = cloned[num].to_vec();
            }

            for post in cloned {
                let thread = Threads::new()?;
                let req =
                    task::spawn(async move { thread.fetch_post(post.as_str()).await }).await??;
                if let Some(resp) = req {
                    posts.push(resp)
                }
            }
        }

        Ok(posts)
    }

    /// Fetches the post's replies
    pub async fn fetch_replies(&self, limit: Option<Range<usize>>) -> Result<Vec<Post>> {
        let mut posts: Vec<Post> = vec![];

        if let Some(ids) = &self.replies {
            let mut cloned = ids.clone();

            if let Some(num) = limit {
                if num.clone().into_iter().any(|x| x >= cloned.len()) {
                    return Err(Error::msg("range is out of bounds"));
                }

                cloned = cloned[num].to_vec();
            }

            for post in cloned {
                let thread = Threads::new()?;
                let req =
                    task::spawn(async move { thread.fetch_post(post.as_str()).await }).await??;
                if let Some(resp) = req {
                    posts.push(resp)
                }
            }
        }

	Ok(posts)
    }
}
