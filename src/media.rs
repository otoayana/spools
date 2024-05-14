use serde::{Deserialize, Serialize};

/// Whether media is image or video
#[derive(Debug, Serialize, Deserialize)]
pub enum MediaKind {
    Image,
    Video,
}

/// Media location and metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub kind: MediaKind,
    pub alt: Option<String>,
    pub content: String,
    pub thumbnail: Option<String>,
}
