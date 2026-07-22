//! Provider-neutral domain types and contracts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Artist,
    Album,
    Track,
    Artwork,
    Playlist,
}

/// Collision-safe media identity. Its parts remain separate in JSON and storage.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaId {
    pub provider_id: ProviderId,
    pub kind: MediaKind,
    pub item_id: String,
}

impl MediaId {
    pub fn new(provider_id: ProviderId, kind: MediaKind, item_id: impl Into<String>) -> Self {
        Self {
            provider_id,
            kind,
            item_id: item_id.into(),
        }
    }
}

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
pub struct ArtistDetail {
    pub artist: Artist,
    pub albums: Vec<Album>,
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
pub struct LibraryAlbum {
    pub id: MediaId,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<MediaId>,
    pub cover_art: Option<MediaId>,
    pub song_count: Option<u32>,
    pub duration_seconds: Option<u64>,
    pub year: Option<u32>,
    pub source_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub owner: Option<String>,
    pub song_count: Option<u32>,
    pub duration_seconds: Option<u64>,
    pub cover_art: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDetail {
    #[serde(flatten)]
    pub playlist: Playlist,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryArtist {
    pub id: MediaId,
    pub name: String,
    pub album_count: Option<u32>,
    pub cover_art: Option<MediaId>,
    pub source_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryArtistDetail {
    pub artist: LibraryArtist,
    pub albums: Vec<LibraryAlbum>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTrack {
    pub id: MediaId,
    pub title: String,
    pub artist: Option<String>,
    pub artist_id: Option<MediaId>,
    pub album: Option<String>,
    pub album_id: Option<MediaId>,
    pub cover_art: Option<MediaId>,
    pub duration_seconds: Option<u64>,
    pub track_number: Option<u32>,
    pub suffix: Option<String>,
    pub source_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryAlbumDetail {
    pub album: LibraryAlbum,
    pub tracks: Vec<LibraryTrack>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlaylist {
    pub id: MediaId,
    pub name: String,
    pub owner: Option<String>,
    pub song_count: Option<u32>,
    pub duration_seconds: Option<u64>,
    pub cover_art: Option<MediaId>,
    pub source_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlaylistDetail {
    pub playlist: LibraryPlaylist,
    pub tracks: Vec<LibraryTrack>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderIssueKind {
    Unauthorized,
    Unavailable,
    InvalidResponse,
    Unsupported,
    Timeout,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIssue {
    pub provider_id: ProviderId,
    pub provider_name: String,
    pub kind: ProviderIssueKind,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregateResponse<T> {
    pub items: Vec<T>,
    pub issues: Vec<ProviderIssue>,
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapability {
    pub name: String,
    pub versions: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorites {
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub server_version: String,
    pub api_version: String,
    pub provider: String,
    pub open_subsonic: bool,
    pub favorites_supported: bool,
    pub scrobbling_supported: bool,
    pub capabilities_known: bool,
    pub capabilities: Vec<ProviderCapability>,
}

impl ProviderStatus {
    pub fn supports(&self, name: &str, version: u32) -> bool {
        self.capabilities.iter().any(|capability| {
            capability.name.eq_ignore_ascii_case(name) && capability.versions.contains(&version)
        })
    }
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
    #[error("provider does not support {0}")]
    Unsupported(String),
}

#[async_trait]
pub trait MusicProvider: Send + Sync {
    async fn ping(&self) -> Result<ProviderStatus, ProviderError>;
    async fn artists(&self, limit: u32, offset: u32) -> Result<Vec<Artist>, ProviderError>;
    async fn artist(&self, id: &str) -> Result<ArtistDetail, ProviderError>;
    async fn albums(&self, limit: u32, offset: u32) -> Result<Vec<Album>, ProviderError>;
    async fn album(&self, id: &str) -> Result<AlbumDetail, ProviderError>;
    async fn playlists(&self) -> Result<Vec<Playlist>, ProviderError>;
    async fn playlist(&self, id: &str) -> Result<PlaylistDetail, ProviderError>;
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<Track>, ProviderError>;
    async fn favorites(&self) -> Result<Favorites, ProviderError>;
    async fn set_favorite(
        &self,
        kind: MediaKind,
        id: &str,
        favorite: bool,
    ) -> Result<(), ProviderError>;
    async fn scrobble(
        &self,
        track_id: &str,
        submission: bool,
        time_ms: Option<u64>,
    ) -> Result<(), ProviderError>;
    fn stream_url(&self, track_id: &str) -> Result<String, ProviderError>;
    fn cover_art_url(&self, cover_art_id: &str, size: Option<u32>)
    -> Result<String, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_capabilities_are_version_gated() {
        let status = ProviderStatus {
            server_version: "1.2.3".into(),
            api_version: "1.16.1".into(),
            provider: "Example".into(),
            open_subsonic: true,
            favorites_supported: true,
            scrobbling_supported: true,
            capabilities_known: true,
            capabilities: vec![ProviderCapability {
                name: "playbackReport".into(),
                versions: vec![1, 2],
            }],
        };

        assert!(status.supports("PlaybackReport", 2));
        assert!(!status.supports("playbackReport", 3));
        assert!(!status.supports("unknown", 1));
    }

    #[test]
    fn media_ids_are_qualified_by_provider_and_kind() {
        let first = MediaId::new(ProviderId::new("one"), MediaKind::Track, "42");
        let second = MediaId::new(ProviderId::new("two"), MediaKind::Track, "42");
        let album = MediaId::new(ProviderId::new("one"), MediaKind::Album, "42");
        assert_ne!(first, second);
        assert_ne!(first, album);
    }

    #[test]
    fn media_id_json_keeps_identity_parts_separate() {
        let id = MediaId::new(ProviderId::new("provider/a"), MediaKind::Track, "song/42");
        let encoded = serde_json::to_string(&id).unwrap();
        assert!(encoded.contains("providerId"));
        assert!(encoded.contains("itemId"));
        assert_eq!(serde_json::from_str::<MediaId>(&encoded).unwrap(), id);
    }
}
