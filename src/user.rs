use std::ops::Range;

use crate::{Post, Threads};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::task;

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

impl User {
    ///
    pub async fn fetch_posts(&self, limit: Option<Range<usize>>) -> Result<Vec<Post>> {
        let mut posts: Vec<Post> = vec![];

        if let Some(ids) = &self.posts {
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
