use crate::SpoolsError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Whether media is image or video
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MediaKind {
    Image,
    Video,
}

/// Media location and metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Media {
    pub kind: MediaKind,
    pub alt: Option<String>,
    pub content: String,
    pub thumbnail: String,
}

impl Media {
    pub(crate) fn from(object: Value) -> Result<Self, SpoolsError> {
        // Set initial values
        let mut kind = MediaKind::Image;
        let content: String;
        let mut alt: Option<String> = None;
        let thumbnail: String;

        // Locations for media media
        let video_location = object.pointer("/video_versions").unwrap_or(&Value::Null);
        let image_location = object
            .pointer("/image_versions2/candidates")
            .unwrap_or(&Value::Null);

        // Gets the first image in URL, since it's in the highest quality
        let image_array = image_location.as_array().unwrap();

        let main_image = &image_array[0];

        let image = main_image["url"].as_str().to_owned().unwrap().to_string();

        // Gets aspect ratio, to find a thumbnail
        let image_width = main_image.clone()["width"].as_f64().to_owned().unwrap();
        let image_height = main_image.clone()["height"].as_f64().to_owned().unwrap();
        let aspect_ratio = ((image_width / image_height) * 10.0).round();

        let thumbnail_array: Vec<String> = image_array
            .into_iter()
            .filter(|val| {
                aspect_ratio
                    == ((val["width"].as_f64().to_owned().unwrap()
                        / val["height"].as_f64().to_owned().unwrap())
                        * 10.0)
                        .round()
            })
            .map(|val| val["url"].as_str().unwrap().to_string())
            .collect();

        let thumbnail_url = thumbnail_array[thumbnail_array.len()/3].to_owned();

        // Alt text
        if object["accessibility_caption"].is_string() {
            alt = Some(
                object["accessibility_caption"]
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
            thumbnail = thumbnail_url;
        } else {
            content = image;
            thumbnail = thumbnail_url;
        }

        Ok(Media {
            kind,
            alt,
            content,
            thumbnail,
        })
    }
}
