use std::iter::repeat_with;

use crate::{
    error::{SpoolsError, Types},
    media::Media,
    post::{Post, Subpost},
    user::{Author, User},
};
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

// Implement internal trait to ease unwrapping strings
trait ValueString {
    fn clean_string(&self) -> String;
}

impl ValueString for Value {
    fn clean_string(&self) -> String {
        self.as_str().unwrap_or("").to_string()
    }
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
        let lsd: String = repeat_with(fastrand::alphanumeric).take(11).collect();

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
                username: post.pointer("/user/username").unwrap().clean_string(),

                pfp: post
                    .pointer("/user/profile_pic_url")
                    .unwrap()
                    .clean_string(),

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

            // Checks for array types
            let image_location = post
                .pointer("/image_versions2/candidates")
                .unwrap_or(&Value::Null);
            let carousel_location = post.pointer("/carousel_media").unwrap_or(&Value::Null);

            // Define media variables
            let mut media: Vec<Media> = vec![];

            // Check where media could be, if there is any
            if carousel_location.is_array() {
                // Carousel media
                media = carousel_location
                    .as_array()
                    .unwrap()
                    .clone()
                    .iter_mut()
                    .map(|node| Media::from(node.clone()).unwrap())
                    .collect();
            } else if image_location.is_array()
                && !image_location.as_array().unwrap_or(&vec![]).is_empty()
            {
                // Singular media

                media.push(Media::from(post.clone())?)
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
            let error = SpoolsError::deserialize_error(resp);

            return Err(if matches!(error, SpoolsError::InvalidResponse) {
                SpoolsError::NotFound(Types::User)
            } else {
                error
            });
        }

        // Defines empty values
        let mut name: String = String::new();
        let mut pfp: String = String::new();
        let mut bio: String = String::new();
        let mut links: Vec<String> = vec![];

        // These variables need to be fetched as str, otherwise they'll be wrapped in explicit quote marks
        let unquot: Vec<String> = vec!["id", "full_name", "biography"]
            .iter()
            .map(|var| parent[var].clean_string())
            .collect();

        // Fetches profile picture
        let pfp_location = parent
            .pointer("/hd_profile_pic_versions")
            .unwrap_or(&Value::Null);

        // We do this for safety, but if the request was successful, this should go smoothly.
        if let Value::Array(versions) = &pfp_location {
            // Gets the highest quality version of the profile pic
            pfp = versions[versions.len() - 1]["url"].clean_string();
        }

        // Sets name and bio values if applicable
        if !unquot[1].is_empty() {
            name.clone_from(&unquot[1])
        }

        if !unquot[2].is_empty() {
            bio.clone_from(&unquot[2])
        }

        // Executes request to get additional information through the user ID
        let id_var = format!("\"userID\":\"{}\"", unquot[0]);
        let id_resp = cloned.query(&id_var, "25253062544340717").await?;

        // Gets user's bio links
        let links_parent = id_resp
            .pointer("/data/user/bio_links")
            .unwrap_or(&Value::Null);

        if let Value::Array(link_array) = &links_parent {
            links = link_array
                .iter()
                .map(|link| link["url"].clean_string())
                .collect()
        }

        // Executes a request to get the user's posts
        let cloned: Threads = self.clone();
        let post_var = format!("\"userID\":\"{}\"", unquot[0]);
        let post_resp = cloned.query(&post_var, "7357407954367176").await?;

        // Gets user's posts
        let edges = post_resp
            .pointer("/data/mediaData/edges")
            .unwrap_or(&Value::Null);

        let posts: Vec<Subpost> = if let Value::Array(nodes) = &edges {
            nodes
                .iter()
                .map(|node| {
                    let thread_items = node.pointer("/node/thread_items").unwrap();

                    thread_items
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|bit| self.build_subpost(bit).unwrap())
                })
                .flatten()
                .collect()
        } else {
            vec![]
        };

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
        let resp = cloned.query(&variables, "26262423843344977").await?;

        let check = resp.pointer("/data/data/edges");
        let post: Post;

        if let Some(Value::Array(content)) = check {
            // Meta wrapping stuff in arrays -.-
            let subposts: Vec<(Subpost, String)> = content
                .clone()
                .iter_mut()
                .map(|node| {
                    if let Value::Array(thread_items) =
                        &node.pointer("/node/thread_items").unwrap_or(&Value::Null)
                    {
                        thread_items
                            .to_owned()
                            .iter()
                            .map(|post| {
                                let builder = Threads::new().unwrap();

                                let result = builder
                                    .build_subpost(&post)
                                    .map_err(|_| SpoolsError::SubpostError)
                                    .unwrap();

                                let rel = post
                                    .pointer("/post/text_post_app_info/reply_to_author/username")
                                    .unwrap_or(&Value::Null)
                                    .clean_string();

                                (result, rel)
                            })
                            .collect::<Vec<(Subpost, String)>>()
                    } else {
                        vec![]
                    }
                })
                .flatten()
                .collect();

            if let Some(out) = subposts.iter().filter(|post| post.0.code == code).next() {
                let slices: Vec<_> = subposts
                    .split(|out| &out.0.code == code)
                    .collect::<Vec<&[(Subpost, String)]>>();

                let parents = match slices.iter().next() {
                    Some(val) => val.iter().map(|post| post.clone().0).collect(),
                    None => vec![],
                };

                let replies = match slices.iter().last() {
                    Some(val) => val
                        .iter()
                        .filter(|val| val.1 == out.0.author.username)
                        .map(|post| post.clone().0)
                        .collect(),
                    None => vec![],
                };

                post = Post {
                    id,
                    author: out.0.author.to_owned(),
                    date: out.0.date,
                    body: out.0.body.to_owned(),
                    media: out.0.media.to_owned(),
                    likes: out.0.likes.to_owned(),
                    parents,
                    replies,
                }
            } else {
                return Err(SpoolsError::NotFound(Types::Post));
            }
        } else {
            return Err(SpoolsError::deserialize_error(resp));
        }

        Ok(post)
    }
}

