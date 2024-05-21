use crate::{
    error::{SpoolsError, Types},
    media::{Media, MediaKind},
    post::{Post, Subpost},
    user::{Author, User},
};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::{header, Client};
use serde_json::Value;

/// Threads pseudo-client
///
/// All requests to the Threads API are done through [`Threads`] methods, which run the requests
/// through a [`reqwest::Client`] prefilled with the correct headers and keys Threads wants us to
/// comply with.
#[derive(Debug, Clone)]
pub struct Threads {
    client: Client,
}

impl Threads {
    /// Create a new [`Threads`].
    pub fn new() -> Result<Threads, SpoolsError> {
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
                .build()
                .map_err(|_| SpoolsError::ClientError)?,
        })
    }

    /// Send a GraphQL query to Threads and return a JSON document
    async fn query(&self, variables: &str, doc_id: &str) -> Result<Value, SpoolsError> {
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
            .await
            .map_err(SpoolsError::RequestError)?;

        let deser = resp
            .json::<Value>()
            .await
            .map_err(|_| SpoolsError::InvalidResponse)?;

        Ok(deser)
    }

    /// Retrieve post ID from shortcode
    async fn fetch_post_id(&self, code: &str) -> Result<String, SpoolsError> {
        let resp = self
            .client
            .get(format!("https://www.threads.net/post/{}", code))
            .header("Sec-Fetch-Node", "navigate")
            .send()
            .await
            .map_err(SpoolsError::RequestError)?
            .text()
            .await
            .map_err(|_| SpoolsError::InvalidResponse)?;

        // Finds the ID, located in a meta tag containing JSON data
        let id_location = resp.find("post_id");

        if id_location.is_none() {
            return Err(SpoolsError::NotFound(Types::Post));
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

        Ok(id)
    }

    /// Deserialize the JSON query for a post
    fn build_subpost(&self, query: &Value) -> Result<Subpost, SpoolsError> {
        if let Some(post) = query.pointer("/post") {
            let code = post
                .pointer("/code")
                .unwrap()
                .as_str()
                .to_owned()
                .unwrap()
                .to_string();

            let author = Author {
                username: post
                    .pointer("/user/username")
                    .unwrap()
                    .as_str()
                    .to_owned()
                    .unwrap()
                    .to_string(),

                pfp: post
                    .pointer("/user/profile_pic_url")
                    .unwrap()
                    .as_str()
                    .to_owned()
                    .unwrap()
                    .to_string(),

                verified: post
                    .pointer("/user/is_verified")
                    .unwrap()
                    .as_bool()
                    .to_owned()
                    .unwrap(),
            };

            // Get the post's date
            let date = post
                .pointer("/taken_at")
                .unwrap()
                .as_u64()
                .to_owned()
                .unwrap();

            // Get the post's body
            let maybe_req = post.pointer("/caption/text");

            let body: String;

            if let Some(maybe_body) = maybe_req {
                if let Value::String(string) = maybe_body {
                    body = string.as_str().to_owned().to_string();
                } else {
                    return Err(SpoolsError::InvalidResponse);
                }
            } else {
                body = String::new();
            }

            // Locations for singular media
            let video_location = post.pointer("/video_versions").unwrap_or(&Value::Null);
            let image_location = post
                .pointer("/image_versions2/candidates")
                .unwrap_or(&Value::Null);

            // Locations for carousel media
            let carousel_location = post.pointer("/carousel_media").unwrap_or(&Value::Null);

            // Define media variables
            let mut media: Vec<Media> = vec![];

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
                    let node_video_location =
                        node.pointer("/video_versions").unwrap_or(&Value::Null);

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

                    media.push(Media {
                        kind,
                        alt,
                        content,
                        thumbnail,
                    });
                }
            } else if image_location.is_array()
                && !image_location.as_array().unwrap_or(&vec![]).is_empty()
            {
                // Singular media
                // Set initial values
                let mut kind = MediaKind::Image;
                let content: String;
                let mut alt: Option<String> = None;
                let mut thumbnail: Option<String> = None;

                // Gets the first image in URL, since it's in the highest quality
                let image_array = image_location.as_array().unwrap();

                let image = image_array[0]["url"]
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

                media.push(Media {
                    kind,
                    alt,
                    content,
                    thumbnail,
                })
            }

            Ok(Subpost {
                code,
                author,
                date,
                body,
                media,
                likes: post["like_count"].as_u64().unwrap_or(0),
            })
        } else {
            Err(SpoolsError::InvalidResponse)
        }
    }

    /// Fetch user information
    pub async fn fetch_user(&self, tag: &str) -> Result<User, SpoolsError> {
        // Executes request to get user info from the username
        let variables = format!("\"username\":\"{}\"", tag);
        let cloned = self.clone();

        let resp = cloned.query(&variables, "7394812507255098").await?;

        // Gets tree location for value
        let parent = resp
            .pointer("/data/xdt_user_by_username")
            .unwrap_or(&Value::Null);

        if let Value::Null = parent {
            return Err(SpoolsError::NotFound(Types::User));
        }

        // Defines empty values
        let mut name: String = String::new();
        let mut pfp: String = String::new();
        let mut bio: String = String::new();
        let mut links: Vec<String> = vec![];
        let mut posts: Vec<Subpost> = vec![];

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
        if let Value::Array(versions) = &pfp_location {
            // Gets the highest quality version of the profile pic
            pfp = versions[versions.len() - 1]["url"]
                .as_str()
                .to_owned()
                .unwrap()
                .to_string();
        }

        // Sets name and bio values if applicable
        if !unquot[1].is_empty() {
            name.clone_from(&unquot[1])
        }

        if !unquot[2].is_empty() {
            bio.clone_from(&unquot[2])
        }

        // Executes request to get additional information through the user ID
        let cloned = self.clone();
        let id_var = format!("\"userID\":\"{}\"", unquot[0]);
        let id_resp = cloned.query(&id_var, "25253062544340717").await?;

        // Gets user's bio links
        let links_parent = id_resp
            .pointer("/data/user/bio_links")
            .unwrap_or(&Value::Null);

        if let Value::Array(link_array) = &links_parent {
            for link in link_array {
                links.push(link["url"].as_str().to_owned().unwrap().to_string())
            }
        }

        // Executes a request to get the user's posts
        let cloned: Threads = self.clone();
        let post_var = format!("\"userID\":\"{}\"", unquot[0]);
        let post_resp = cloned.query(&post_var, "7357407954367176").await?;

        // Gets user's posts
        let edges = post_resp
            .pointer("/data/mediaData/edges")
            .unwrap_or(&Value::Null);

        if let Value::Array(nodes) = &edges {
            for node in nodes {
                let thread_items = node.pointer("/node/thread_items").unwrap();

                for item in thread_items.as_array().unwrap() {
                    posts.push(self.build_subpost(item)?)
                }
            }
        }

        Ok(User {
            id: unquot[0]
                .parse::<u64>()
                .map_err(|_| SpoolsError::InvalidResponse)?,
            name,
            pfp,
            bio,
            links,
            verified: parent["is_verified"].as_bool().unwrap_or(false),
            followers: parent["follower_count"].as_u64().unwrap_or(0),
            posts,
        })
    }

    /// Fetch post information
    pub async fn fetch_post(&self, code: &str) -> Result<Post, SpoolsError> {
        // Since there's no endpoint for getting full IDs out of short ones, fetch it from post URL
        let inner_code = code.to_owned();
        let cloned = self.clone();
        let id = cloned.fetch_post_id(inner_code.as_str()).await?;

        // Now we can fetch the actual post
        let variables = format!("\"postID\":\"{}\"", &id);
        let cloned: Threads = self.clone();
        let resp = cloned.query(&variables, "26262423843344977").await?;

        let check = resp.pointer("/data/data/edges");
        let post: Post;

        if let Some(content) = check {
            // Defines initial values for parents and replies
            let mut parents: Vec<Subpost> = vec![];
            let mut replies: Vec<Subpost> = vec![];

            // Defines initial values for post location
            let mut subpost: Option<Subpost> = None;

            // Meta wrapping stuff in arrays -.-
            if let Value::Array(node_array) = content {
                for node in node_array {
                    if let Value::Array(thread_items) =
                        &node.pointer("/node/thread_items").unwrap_or(&Value::Null)
                    {
                        for item in thread_items {
                            let builder = Threads::new()?;
                            let cur = builder
                                .build_subpost(item)
                                .map_err(|_| SpoolsError::SubpostError)?;

                            if cur.code == code {
                                subpost = Some(cur);
                            } else if let Some(post) = &subpost {
                                let username_req = &item
                                    .pointer("/post/text_post_app_info/reply_to_author/username")
                                    .unwrap_or(&Value::Null);

                                if let Some(name) = username_req.as_str().to_owned() {
                                    if name == post.author.username {
                                        replies.push(cur);
                                    }
                                }
                            } else {
                                parents.push(cur);
                            }
                        }
                    }
                }

                if let Some(fields) = subpost {
                    post = Post {
                        id,
                        author: fields.author,
                        date: fields.date,
                        body: fields.body,
                        media: fields.media,
                        likes: fields.likes,
                        parents,
                        replies,
                    }
                } else {
                    return Err(SpoolsError::NotFound(Types::Post));
                }
            } else {
                return Err(SpoolsError::NotFound(Types::Post));
            }
        } else {
            return Err(SpoolsError::NotFound(Types::Post));
        }

        Ok(post)
    }
}
