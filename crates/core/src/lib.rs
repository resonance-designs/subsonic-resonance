//! Provider-neutral domain types and contracts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub album_count: Option<u32>,
    pub cover_art: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    pub song_count: Option<u32>,
    pub duration_seconds: Option<u64>,
    pub year: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub cover_art: Option<String>,
    pub duration_seconds: Option<u64>,
    pub track_number: Option<u32>,
    pub suffix: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetail {
    #[serde(flatten)]
    pub album: Album,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub server_version: String,
    pub provider: String,
    pub open_subsonic: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("authentication failed")]
    Unauthorized,
    #[error("resource not found")]
    NotFound,
    #[error("provider returned error {code}: {message}")]
    Remote { code: i32, message: String },
    #[error("provider response was invalid: {0}")]
    InvalidResponse(String),
    #[error("provider is unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait MusicProvider: Send + Sync {
    async fn ping(&self) -> Result<ProviderStatus, ProviderError>;
    async fn albums(&self, limit: u32, offset: u32) -> Result<Vec<Album>, ProviderError>;
    async fn album(&self, id: &str) -> Result<AlbumDetail, ProviderError>;
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<Track>, ProviderError>;
    fn stream_url(&self, track_id: &str) -> Result<String, ProviderError>;
    fn cover_art_url(&self, cover_art_id: &str, size: Option<u32>)
    -> Result<String, ProviderError>;
}
