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
//! # use anyhow::Result;
//! #
//! # async fn run() -> Result<()> {
//! let client = spools::Threads::new()?;
//! let user = client.fetch_user("zuck")?;
//! if let Some(posts) = user.unwrap().posts {
//!     let post = client.fetch_post(&posts[0])?;
//! };
//! #     Ok(())
//! # }

#[cfg(test)]
mod test;

use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub kind: MediaKind,
    pub alt: Option<String>,
    pub content: String,
    pub thumbnail: Option<String>,
}

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

/// Threads pseudo-client
#[derive(Debug, Clone)]
pub struct Threads {
    client: Client,
}

impl Threads {
    /// Create a new [`Threads`].
    pub fn new() -> Result<Threads> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Sec-Fetch-Site",
            header::HeaderValue::from_static("same-origin"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("x-www-form-urlencoded"),
        );

        Ok(Threads {
            client: Client::builder()
                .default_headers(headers)
                .user_agent(
                    "Mozilla/5.0 (X11; Linux x86_64; rv:125.0) Gecko/20100101 Firefox/125.0",
                )
                .build()?,
        })
    }

    /// Send a GraphQL query to Threads and return a JSON document
    #[tokio::main]
    async fn query(&self, variables: &str, doc_id: &str) -> Result<Value> {
        // Meta uses 11 characters, though 12 also works
        let lsd = Alphanumeric.sample_string(&mut rand::thread_rng(), 11);

        let params = [
            ("lsd", lsd.as_str()),
            ("variables", &format!("{{{},\"__relay_internal__pv__BarcelonaIsLoggedInrelayprovider\":false,\"__relay_internal__pv__BarcelonaIsOriginalPostPillEnabledrelayprovider\":false,\"__relay_internal__pv__BarcelonaIsThreadContextHeaderEnabledrelayprovider\":false,
    	\"__relay_internal__pv__BarcelonaIsSableEnabledrelayprovider\":false,\"__relay_internal__pv__BarcelonaUseCometVideoPlaybackEnginerelayprovider\":false,\"__relay_internal__pv__BarcelonaOptionalCookiesEnabledrelayprovider\":true,\"__relay_internal__pv__BarcelonaShouldShowFediverseM075Featuresrelayprovider\":false}}", variables)),
            ("doc_id", doc_id),
        ];

        let resp = self
            .client
            .post("https://www.threads.net/api/graphql")
            .form(&params)
            .header("X-FB-LSD", lsd)
            .send()
            .await?;

        let deser = resp.json::<Value>().await?;
        Ok(deser)
    }

    /// Retrieve full post ID from short ID
    #[tokio::main]
    async fn full_id(&self, id: &str) -> Result<Option<String>> {
        let resp = self
            .client
            .get(format!("https://www.threads.net/post/{}", id))
            .header("Sec-Fetch-Node", "navigate")
            .send()
            .await?
            .text()
            .await?;

        // Finds the ID, located in a meta tag containing JSON data
        let id_location = resp.find("post_id");
        if id_location.is_none() {
            return Ok(None);
        }

        // Prepare values to select the ID
        let mut cur = id_location.unwrap() + 10;
        let mut curchar = resp.as_bytes()[cur] as char;
        let mut id = String::new();

        while curchar != '\"' {
            id.push(curchar);
            cur += 1;
            curchar = resp.as_bytes()[cur] as char;
        }

        Ok(Some(id))
    }

    /// Fetch user information
    #[tokio::main]
    pub async fn fetch_user(&self, tag: &str) -> Result<Option<User>> {
        // Executes request to get user info from the username
        let variables: String = format!("\"username\":\"{}\"", tag);
        let cloned = self.clone();
        let resp =
            task::spawn_blocking(move || cloned.query(&variables, "7394812507255098")).await??;

        // Gets tree location for value
        let parent = resp
            .pointer("/data/xdt_user_by_username")
            .unwrap_or(&Value::Null);

        if parent.is_null() {
            return Ok(None);
        }

        // Defines empty values
        let mut name: Option<String> = None;
        let mut pfp: Option<String> = None;
        let mut bio: Option<String> = None;
        let mut links: Option<Vec<String>> = None;
        let mut posts: Option<Vec<String>> = None;

        // These variables need to be fetched as str, otherwise they'll be wrapped in explicit quote marks
        let quot = vec!["id", "full_name", "biography"];
        let mut unquot: Vec<String> = vec![];

        for val in quot {
            unquot.push(parent[val].as_str().to_owned().unwrap().to_string())
        }

        // Fetches profile picture
        let pfp_location = parent
            .pointer("/hd_profile_pic_versions")
            .unwrap_or(&Value::Null);

        // We do this for safety, but if the request was successful, this should go smoothly.
        if pfp_location.is_array() {
            let pfp_versions = pfp_location.as_array().unwrap();

            // Gets the highest quality version of the profile pic
            pfp = Some(
                pfp_versions[pfp_versions.len() - 1]["url"]
                    .as_str()
                    .to_owned()
                    .unwrap()
                    .to_string(),
            );
        }

        // Sets name and bio values if applicable
        if !unquot[1].is_empty() {
            name = Some(unquot[1].clone())
        }

        if !unquot[2].is_empty() {
            bio = Some(unquot[2].clone())
        }

        // Executes request to get additional information through the user ID
        let cloned = self.clone();
        let id_var = format!("\"userID\":\"{}\"", unquot[0]);
        let id_resp =
            task::spawn_blocking(move || cloned.query(&id_var, "25253062544340717")).await??;

        // Gets user's bio links
        let links_parent = id_resp
            .pointer("/data/user/bio_links")
            .unwrap_or(&Value::Null);

        if links_parent.is_array() {
            let mut links_vec: Vec<String> = vec![];
            for x in links_parent.as_array().unwrap() {
                links_vec.push(x["url"].as_str().to_owned().unwrap().to_string())
            }
            links = Some(links_vec);
        }

        // Executes a request to get the user's posts
        let cloned = self.clone();
        let post_var = format!("\"userID\":\"{}\"", unquot[0]);
        let post_resp =
            task::spawn_blocking(move || cloned.query(&post_var, "7357407954367176")).await??;

        // Gets users' posts
        let edges = post_resp
            .pointer("/data/mediaData/edges")
            .unwrap_or(&Value::Null);
        if edges.is_array() {
            let node_array = edges.as_array().unwrap();
            let mut post_vec: Vec<String> = vec![];
            for node in node_array {
                let thread_items = node.pointer("/node/thread_items").unwrap();
                for item in thread_items.as_array().unwrap() {
                    let cur = item.pointer("/post").unwrap();
                    let code = cur["code"].as_str().to_owned().unwrap();
                    post_vec.push(code.to_string());
                }
            }
            posts = Some(post_vec);
        }

        Ok(Some(User {
            id: unquot[0].parse::<u64>()?,
            name,
            pfp,
            bio,
            links,
            verified: parent["is_verified"].as_bool().unwrap_or(false),
            followers: parent["follower_count"].as_u64().unwrap_or(0),
            posts,
        }))
    }

    /// Fetch post information
    #[tokio::main]
    pub async fn fetch_post(&self, id: &str) -> Result<Option<Post>> {
        // Since there's no endpoint for getting full IDs out of short ones, fetch it from post URL
        let inner_id = id.to_owned();
        let cloned = self.clone();
        let id_req = task::spawn_blocking(move || cloned.full_id(&inner_id)).await??;

        if id_req.is_none() {
            return Ok(None);
        }

        let fullid = id_req.unwrap_or(String::new());

        // Now we can fetch the actual post
        let variables = format!("\"postID\":\"{}\"", &fullid);
        let cloned = self.clone();
        let resp =
            task::spawn_blocking(move || cloned.query(&variables, "26262423843344977")).await??;

        let check = resp.pointer("/data/data/edges");

        if check.is_none() {
            return Ok(None);
        }

        // Defines values for parents and replies
        let mut parents: Option<Vec<String>> = None;
        let mut replies: Option<Vec<String>> = None;

        let mut parents_vec: Vec<String> = vec![];
        let mut replies_vec: Vec<String> = vec![];

        // Defines values for post location
        let mut post = &Value::Null;
        let mut post_found: bool = false;

        // Meta wrapping stuff in arrays -.-
        let node_array = check.unwrap_or(&Value::Null).as_array().unwrap();

        for node in node_array {
            let thread_items = node.pointer("/node/thread_items").unwrap_or(&Value::Null);

            if !thread_items.is_array() {
                return Ok(None);
            }

            for item in thread_items.as_array().unwrap() {
                let cur = item.pointer("/post").unwrap();
                let code = cur["code"].as_str().to_owned().unwrap();
                if code == id {
                    post = cur;
                    post_found = true;
                } else if !post_found {
                    parents_vec.push(code.to_string());
                    parents = Some(parents_vec.clone());
                } else {
                    replies_vec.push(code.to_string());
                    replies = Some(replies_vec.clone());
                }
            }
        }

        // Get the post's author
        let tag = post
            .pointer("/user/username")
            .unwrap()
            .as_str()
            .to_owned()
            .unwrap();

        // Get the post's date
        let date = post
            .pointer("/taken_at")
            .unwrap()
            .as_u64()
            .to_owned()
            .unwrap();

        // Get the post's body
        let body = post
            .pointer("/caption/text")
            .unwrap()
            .as_str()
            .to_owned()
            .unwrap();

        // Locations for singular media
        let video_location = post.pointer("/video_versions").unwrap_or(&Value::Null);
        let image_location = post
            .pointer("/image_versions2/candidates")
            .unwrap_or(&Value::Null);

        // Locations for carousel media
        let carousel_location = post.pointer("/carousel_media").unwrap_or(&Value::Null);

        // Define media variables
        let mut media: Option<Vec<Media>> = None;
        let mut media_vec: Vec<Media> = vec![];

        // Check where media could be, if there is any
        if carousel_location.is_array() {
            // Carousel media
            let carousel_array = carousel_location.as_array().unwrap();
            for node in carousel_array {
                // Initial values
                let mut kind = MediaKind::Image;
                let content: String;
                let mut alt: Option<String> = None;
                let mut thumbnail: Option<String> = None;

                // Image
                let node_image_location = &node
                    .pointer("/image_versions2/candidates")
                    .unwrap()
                    .as_array()
                    .unwrap()[0];
                let node_video_location = node.pointer("/video_versions").unwrap_or(&Value::Null);

                // CDN URL
                let image_url = node_image_location["url"]
                    .as_str()
                    .to_owned()
                    .unwrap()
                    .to_string();

                // Alt text
                if !node["accessibility_caption"].is_null() {
                    alt = Some(
                        node["accessibility_caption"]
                            .as_str()
                            .to_owned()
                            .unwrap()
                            .to_string(),
                    );
                }

                let image = image_url.clone();

                // Video
                if node_video_location.is_array() {
                    let video_array = node_video_location.as_array().unwrap();

                    let video = video_array[0]["url"]
                        .as_str()
                        .to_owned()
                        .unwrap()
                        .to_string();

                    kind = MediaKind::Video;
                    content = video;
                    thumbnail = Some(image);
                } else {
                    content = image;
                }

                media_vec.push(Media {
                    kind,
                    alt,
                    content,
                    thumbnail,
                });
            }
        } else if image_location.is_array()
            && image_location.as_array().unwrap_or(&vec![]).len() != 0
        {
            // Singular media
            // Initial values
            let mut kind = MediaKind::Image;
            let content: String;
            let mut alt: Option<String> = None;
            let mut thumbnail: Option<String> = None;

            // Gets the first image in URL, since it's in the highest quality
            let image_array = image_location.as_array().unwrap();

            let image_url = image_array[0]["url"]
                .as_str()
                .to_owned()
                .unwrap()
                .to_string();

            // Alt text
            if post["accessibility_caption"].is_string() {
                alt = Some(
                    post["accessibility_caption"]
                        .as_str()
                        .to_owned()
                        .unwrap()
                        .to_string(),
                );
            }

            let image = image_url.clone();

            // Video
            if video_location.is_array() {
                let video_array = video_location.as_array().unwrap();
                let video = video_array[0]["url"]
                    .as_str()
                    .to_owned()
                    .unwrap()
                    .to_string();

                kind = MediaKind::Video;
                content = video;
                thumbnail = Some(image);
            } else {
                content = image;
            }

            media_vec.push(Media {
                kind,
                alt,
                content,
                thumbnail,
            })
        }

        // If there was media, we add it to the response.
        if media_vec.len() != 0 {
            media = Some(media_vec);
        }

        Ok(Some(Post {
            id: fullid,
            name: tag.to_string(),
            date,
            body: Some(body.to_string()),
            media,
            likes: post["like_count"].as_u64().unwrap_or(0),
            reposts: post
                .pointer("/text_post_app_info/repost_count")
                .unwrap()
                .as_u64()
                .unwrap_or(0),
            parents,
            replies,
        }))
    }
}
