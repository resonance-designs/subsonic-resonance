//! OpenSubsonic/Subsonic provider adapter.

use async_trait::async_trait;
use md5::{Digest, Md5};
use rand::{Rng, distr::Alphanumeric};
use reqwest::Client;
use resonance_core::{
    Album, AlbumDetail, Artist, ArtistDetail, MusicProvider, Playlist, PlaylistDetail,
    ProviderError, ProviderStatus, Track,
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use url::Url;

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "subsonic-resonance";

#[derive(Clone, Debug)]
pub enum Credentials {
    ApiKey(String),
    Password { username: String, password: String },
}

#[derive(Clone)]
pub struct SubsonicClient {
    base_url: Url,
    credentials: Credentials,
    http: Client,
}

impl SubsonicClient {
    pub fn new(base_url: impl AsRef<str>, credentials: Credentials) -> Result<Self, ProviderError> {
        let mut base_url = Url::parse(base_url.as_ref()).map_err(|error| {
            ProviderError::InvalidResponse(format!("invalid server URL: {error}"))
        })?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self {
            base_url,
            credentials,
            http: Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|error| ProviderError::Unavailable(error.to_string()))?,
        })
    }

    fn endpoint(&self, method: &str, arguments: &[(&str, String)]) -> Result<Url, ProviderError> {
        let mut url = self
            .base_url
            .join(&format!("rest/{method}"))
            .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("v", API_VERSION)
                .append_pair("c", CLIENT_NAME)
                .append_pair("f", "json");
            match &self.credentials {
                Credentials::ApiKey(key) => {
                    query.append_pair("apiKey", key);
                }
                Credentials::Password { username, password } => {
                    let salt: String = rand::rng()
                        .sample_iter(&Alphanumeric)
                        .take(16)
                        .map(char::from)
                        .collect();
                    let token = hex::encode(Md5::digest(format!("{password}{salt}").as_bytes()));
                    query
                        .append_pair("u", username)
                        .append_pair("s", &salt)
                        .append_pair("t", &token);
                }
            }
            for (key, value) in arguments {
                query.append_pair(key, value);
            }
        }
        Ok(url)
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        arguments: &[(&str, String)],
    ) -> Result<ApiResult<T>, ProviderError> {
        let response = self
            .http
            .get(self.endpoint(method, arguments)?)
            .send()
            .await
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;
        let http_status = response.status();
        if http_status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Unauthorized);
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;
        let envelope: Value = serde_json::from_slice(&bytes).map_err(|error| {
            ProviderError::InvalidResponse(format!(
                "{method} returned HTTP {http_status} ({content_type}), not valid JSON: {error}"
            ))
        })?;
        let response_value = envelope.get("subsonic-response").cloned().ok_or_else(|| {
            ProviderError::InvalidResponse(format!(
                "{method} response did not contain subsonic-response"
            ))
        })?;
        let metadata: ResponseMetadata =
            serde_json::from_value(response_value.clone()).map_err(|error| {
                ProviderError::InvalidResponse(format!(
                    "{method} response metadata was invalid: {error}"
                ))
            })?;
        if metadata.status != "ok" {
            let error = metadata.error.unwrap_or(ApiError {
                code: 0,
                message: "unknown server error".into(),
            });
            return Err(if error.code == 40 {
                ProviderError::Unauthorized
            } else {
                ProviderError::Remote {
                    code: error.code,
                    message: error.message,
                }
            });
        }
        let body: T = serde_json::from_value(response_value).map_err(|error| {
            ProviderError::InvalidResponse(format!("{method} response schema was invalid: {error}"))
        })?;
        Ok(ApiResult {
            body,
            version: metadata.version,
            server_type: metadata.server_type,
            open_subsonic: metadata.open_subsonic,
        })
    }

    fn missing_result(error: &ProviderError) -> bool {
        matches!(error, ProviderError::InvalidResponse(message) if message.contains("missing field") || message.contains("did not contain its result"))
    }

    fn artist_detail_compatibility_error(error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::InvalidResponse(_) | ProviderError::Remote { .. }
        )
    }

    async fn album_page(&self, size: u32, offset: u32) -> Result<Vec<Album>, ProviderError> {
        let arguments = [
            ("type", "newest".into()),
            ("size", size.to_string()),
            ("offset", offset.to_string()),
        ];
        let modern: Result<ApiResult<AlbumListBody>, ProviderError> =
            self.get("getAlbumList2", &arguments).await;
        let items = match modern {
            Ok(response) => response.body.album_list.album,
            Err(error) if Self::missing_result(&error) => {
                let legacy: ApiResult<LegacyAlbumListBody> =
                    self.get("getAlbumList", &arguments).await?;
                legacy.body.album_list.album
            }
            Err(error) => return Err(error),
        };
        Ok(items.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl MusicProvider for SubsonicClient {
    async fn ping(&self) -> Result<ProviderStatus, ProviderError> {
        let response: ApiResult<PingBody> = self.get("ping", &[]).await?;
        Ok(ProviderStatus {
            server_version: response.version,
            provider: response.server_type.unwrap_or_else(|| "Subsonic".into()),
            open_subsonic: response.open_subsonic.unwrap_or(false),
        })
    }

    async fn artists(&self, limit: u32, offset: u32) -> Result<Vec<Artist>, ProviderError> {
        let modern: Result<ApiResult<ArtistsBody>, ProviderError> =
            self.get("getArtists", &[]).await;
        let mut artists = match modern {
            Ok(response) => response
                .body
                .artists
                .index
                .into_iter()
                .flat_map(|index| index.artist)
                .map(Into::into)
                .collect::<Vec<_>>(),
            Err(error) if Self::missing_result(&error) => {
                let mut derived = HashMap::<String, Artist>::new();
                for album in self.albums(500, 0).await? {
                    let Some(name) = album.artist else { continue };
                    let id = album.artist_id.unwrap_or_else(|| format!("name:{name}"));
                    let artist = derived.entry(id.clone()).or_insert_with(|| Artist {
                        id,
                        name,
                        album_count: Some(0),
                        cover_art: album.cover_art.clone(),
                    });
                    artist.album_count = Some(artist.album_count.unwrap_or(0).saturating_add(1));
                    if artist.cover_art.is_none() {
                        artist.cover_art = album.cover_art;
                    }
                }
                derived.into_values().collect()
            }
            Err(error) => return Err(error),
        };
        artists.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(artists
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn artist(&self, id: &str) -> Result<ArtistDetail, ProviderError> {
        let response: Result<ApiResult<ArtistBody>, ProviderError> =
            self.get("getArtist", &[("id", id.into())]).await;
        match response {
            Ok(response) => {
                let response_artist: Artist = response.body.artist.artist.into();
                let response_albums: Vec<Album> = response
                    .body
                    .artist
                    .album
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let has_attribution = albums_have_artist_attribution(&response_albums);
                let detail = scoped_artist_detail(response_artist.clone(), response_albums);
                let count_conflicts = response_artist
                    .album_count
                    .is_some_and(|expected| detail.albums.len() != expected as usize);

                if !has_attribution || count_conflicts {
                    let artist = self
                        .artists(u32::MAX, 0)
                        .await
                        .ok()
                        .and_then(|artists| artists.into_iter().find(|artist| artist.id == id))
                        .unwrap_or(response_artist);
                    if let Ok(albums) = self.albums(500, 0).await {
                        return Ok(scoped_artist_detail(artist, albums));
                    }
                }
                Ok(detail)
            }
            Err(error) if Self::artist_detail_compatibility_error(&error) => {
                let artists = self.artists(u32::MAX, 0).await?;
                let albums = self.albums(500, 0).await?;
                artist_detail_from_catalog(id, artists, albums).ok_or(error)
            }
            Err(error) => Err(error),
        }
    }

    async fn albums(&self, limit: u32, offset: u32) -> Result<Vec<Album>, ProviderError> {
        const PAGE_SIZE: u32 = 500;
        let target = limit.min(500);
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        let mut page_offset = offset;

        while (items.len() as u32) < target {
            let request_size = PAGE_SIZE.min(target.saturating_sub(items.len() as u32));
            let page = self.album_page(request_size, page_offset).await?;
            if page.is_empty() {
                break;
            }

            let mut added = 0usize;
            for album in page {
                if seen.insert(album.id.clone()) {
                    items.push(album);
                    added += 1;
                }
            }

            // Some partial implementations ignore offset and repeat the first page.
            if added == 0 {
                break;
            }
            page_offset = page_offset.saturating_add(request_size);
        }

        items.truncate(target as usize);
        Ok(items)
    }

    async fn album(&self, id: &str) -> Result<AlbumDetail, ProviderError> {
        let modern: Result<ApiResult<AlbumBody>, ProviderError> =
            self.get("getAlbum", &[("id", id.into())]).await;
        match modern {
            Ok(response) => {
                let album = Album::from(response.body.album.clone());
                Ok(AlbumDetail {
                    album,
                    tracks: response
                        .body
                        .album
                        .song
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            Err(error) if Self::missing_result(&error) => {
                let legacy: ApiResult<MusicDirectoryBody> =
                    self.get("getMusicDirectory", &[("id", id.into())]).await?;
                let directory = legacy.body.directory;
                let tracks: Vec<Track> = directory
                    .child
                    .into_iter()
                    .filter(|item| !item.is_dir.unwrap_or(false))
                    .map(Into::into)
                    .collect();
                let first = tracks.first();
                Ok(AlbumDetail {
                    album: Album {
                        id: directory.id,
                        name: directory.name,
                        artist: first.and_then(|track| track.artist.clone()),
                        artist_id: first.and_then(|track| track.artist_id.clone()),
                        cover_art: first.and_then(|track| track.cover_art.clone()),
                        song_count: Some(tracks.len() as u32),
                        duration_seconds: Some(
                            tracks
                                .iter()
                                .filter_map(|track| track.duration_seconds)
                                .sum(),
                        ),
                        year: None,
                    },
                    tracks,
                })
            }
            Err(error) => Err(error),
        }
    }

    async fn playlists(&self) -> Result<Vec<Playlist>, ProviderError> {
        let response: ApiResult<PlaylistsBody> = self.get("getPlaylists", &[]).await?;
        let mut playlists = response
            .body
            .playlists
            .playlist
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Playlist>>();
        playlists.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(playlists)
    }

    async fn playlist(&self, id: &str) -> Result<PlaylistDetail, ProviderError> {
        let response: ApiResult<PlaylistBody> =
            self.get("getPlaylist", &[("id", id.into())]).await?;
        let mut playlist = response.body.playlist;
        let tracks = std::mem::take(&mut playlist.entry)
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(PlaylistDetail {
            playlist: playlist.into(),
            tracks,
        })
    }

    async fn search(&self, query: &str, limit: u32) -> Result<Vec<Track>, ProviderError> {
        let response: ApiResult<SearchBody> = self
            .get(
                "search3",
                &[
                    ("query", query.into()),
                    ("songCount", limit.min(500).to_string()),
                    ("albumCount", "0".into()),
                    ("artistCount", "0".into()),
                ],
            )
            .await?;
        Ok(response
            .body
            .search_result
            .song
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn stream_url(&self, track_id: &str) -> Result<String, ProviderError> {
        Ok(self.endpoint("stream", &[("id", track_id.into())])?.into())
    }
    fn cover_art_url(&self, id: &str, size: Option<u32>) -> Result<String, ProviderError> {
        let mut args = vec![("id", id.into())];
        if let Some(size) = size {
            args.push(("size", size.to_string()));
        }
        Ok(self.endpoint("getCoverArt", &args)?.into())
    }
}

#[derive(Deserialize)]
struct ResponseMetadata {
    status: String,
    version: String,
    #[serde(rename = "type")]
    server_type: Option<String>,
    #[serde(rename = "openSubsonic")]
    open_subsonic: Option<bool>,
    error: Option<ApiError>,
}
struct ApiResult<T> {
    body: T,
    version: String,
    server_type: Option<String>,
    open_subsonic: Option<bool>,
}
#[derive(Deserialize)]
struct ApiError {
    code: i32,
    message: String,
}

#[derive(Deserialize)]
struct PingBody {}

#[derive(Deserialize)]
struct ArtistsBody {
    artists: SubsonicArtists,
}

#[derive(Deserialize)]
struct ArtistBody {
    artist: SubsonicArtistDetail,
}

#[derive(Deserialize)]
struct SubsonicArtistDetail {
    #[serde(flatten)]
    artist: SubsonicArtist,
    #[serde(default)]
    album: Vec<SubsonicAlbum>,
}

#[derive(Deserialize)]
struct SubsonicArtists {
    #[serde(default)]
    index: Vec<ArtistIndex>,
}

#[derive(Deserialize)]
struct ArtistIndex {
    #[serde(default)]
    artist: Vec<SubsonicArtist>,
}

#[derive(Deserialize)]
struct SubsonicArtist {
    #[serde(deserialize_with = "string_from_any")]
    id: String,
    name: String,
    #[serde(rename = "albumCount")]
    #[serde(default, deserialize_with = "option_u32_from_any")]
    album_count: Option<u32>,
    #[serde(rename = "coverArt")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    cover_art: Option<String>,
}

#[derive(Deserialize)]
struct AlbumListBody {
    #[serde(rename = "albumList2")]
    album_list: AlbumList,
}
#[derive(Deserialize)]
struct LegacyAlbumListBody {
    #[serde(rename = "albumList")]
    album_list: AlbumList,
}
#[derive(Deserialize)]
struct AlbumList {
    #[serde(default)]
    album: Vec<SubsonicAlbum>,
}
#[derive(Clone, Deserialize)]
struct SubsonicAlbum {
    #[serde(deserialize_with = "string_from_any")]
    id: String,
    #[serde(alias = "title", alias = "album")]
    name: String,
    artist: Option<String>,
    #[serde(rename = "artistId")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    artist_id: Option<String>,
    #[serde(rename = "coverArt")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    cover_art: Option<String>,
    #[serde(rename = "songCount")]
    #[serde(default, deserialize_with = "option_u32_from_any")]
    song_count: Option<u32>,
    #[serde(default, deserialize_with = "option_u64_from_any")]
    duration: Option<u64>,
    #[serde(default, deserialize_with = "option_u32_from_any")]
    year: Option<u32>,
    #[serde(default)]
    song: Vec<SubsonicTrack>,
}
#[derive(Deserialize)]
struct AlbumBody {
    album: SubsonicAlbum,
}

#[derive(Deserialize)]
struct PlaylistsBody {
    playlists: PlaylistCollection,
}

#[derive(Deserialize)]
struct PlaylistCollection {
    #[serde(default)]
    playlist: Vec<SubsonicPlaylist>,
}

#[derive(Deserialize)]
struct PlaylistBody {
    playlist: SubsonicPlaylist,
}

#[derive(Clone, Deserialize)]
struct SubsonicPlaylist {
    #[serde(deserialize_with = "string_from_any")]
    id: String,
    name: String,
    owner: Option<String>,
    #[serde(rename = "songCount")]
    #[serde(default, deserialize_with = "option_u32_from_any")]
    song_count: Option<u32>,
    #[serde(default, deserialize_with = "option_u64_from_any")]
    duration: Option<u64>,
    #[serde(rename = "coverArt")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    cover_art: Option<String>,
    #[serde(default)]
    entry: Vec<SubsonicTrack>,
}
#[derive(Deserialize)]
struct SearchBody {
    #[serde(rename = "searchResult3")]
    search_result: SearchResult,
}
#[derive(Deserialize)]
struct SearchResult {
    #[serde(default)]
    song: Vec<SubsonicTrack>,
}
#[derive(Deserialize)]
struct MusicDirectoryBody {
    directory: MusicDirectory,
}
#[derive(Deserialize)]
struct MusicDirectory {
    #[serde(deserialize_with = "string_from_any")]
    id: String,
    name: String,
    #[serde(default)]
    child: Vec<SubsonicTrack>,
}
#[derive(Clone, Deserialize)]
struct SubsonicTrack {
    #[serde(deserialize_with = "string_from_any")]
    id: String,
    title: String,
    artist: Option<String>,
    #[serde(rename = "artistId")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    artist_id: Option<String>,
    album: Option<String>,
    #[serde(rename = "albumId")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    album_id: Option<String>,
    #[serde(rename = "coverArt")]
    #[serde(default, deserialize_with = "option_string_from_any")]
    cover_art: Option<String>,
    #[serde(default, deserialize_with = "option_u64_from_any")]
    duration: Option<u64>,
    #[serde(default, deserialize_with = "option_u32_from_any")]
    track: Option<u32>,
    suffix: Option<String>,
    #[serde(rename = "isDir")]
    is_dir: Option<bool>,
}

fn string_from_any<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        value => Err(serde::de::Error::custom(format!(
            "expected string or number, got {value}"
        ))),
    }
}

fn option_string_from_any<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        Some(value) => Err(serde::de::Error::custom(format!(
            "expected string, number, or null, got {value}"
        ))),
    }
}

fn option_u64_from_any<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u64>, D::Error> {
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => {
            if let Some(unsigned) = value.as_u64() {
                Ok(Some(unsigned))
            } else if value.as_i64().is_some_and(|signed| signed < 0) {
                Ok(None)
            } else if let Some(float) = value.as_f64().filter(|float| *float >= 0.0) {
                Ok(Some(float as u64))
            } else {
                Err(serde::de::Error::custom("expected numeric metadata"))
            }
        }
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(Value::String(value)) => {
            if let Ok(signed) = value.parse::<i64>() {
                Ok((signed >= 0).then_some(signed as u64))
            } else if let Ok(float) = value.parse::<f64>() {
                Ok((float >= 0.0).then_some(float as u64))
            } else {
                Err(serde::de::Error::custom(format!(
                    "expected numeric metadata, got {value}"
                )))
            }
        }
        Some(value) => Err(serde::de::Error::custom(format!(
            "expected integer, string, or null, got {value}"
        ))),
    }
}

fn option_u32_from_any<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    option_u64_from_any(deserializer)?
        .map(|value| u32::try_from(value).map_err(serde::de::Error::custom))
        .transpose()
}

impl From<SubsonicAlbum> for Album {
    fn from(a: SubsonicAlbum) -> Self {
        Self {
            id: a.id,
            name: a.name,
            artist: a.artist,
            artist_id: a.artist_id,
            cover_art: a.cover_art,
            song_count: a.song_count,
            duration_seconds: a.duration,
            year: a.year,
        }
    }
}
impl From<SubsonicArtist> for Artist {
    fn from(artist: SubsonicArtist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            album_count: artist.album_count,
            cover_art: artist.cover_art,
        }
    }
}
impl From<SubsonicPlaylist> for Playlist {
    fn from(playlist: SubsonicPlaylist) -> Self {
        Self {
            id: playlist.id,
            name: playlist.name,
            owner: playlist.owner,
            song_count: playlist.song_count,
            duration_seconds: playlist.duration,
            cover_art: playlist.cover_art,
        }
    }
}
impl From<SubsonicTrack> for Track {
    fn from(t: SubsonicTrack) -> Self {
        Self {
            id: t.id,
            title: t.title,
            artist: t.artist,
            artist_id: t.artist_id,
            album: t.album,
            album_id: t.album_id,
            cover_art: t.cover_art,
            duration_seconds: t.duration,
            track_number: t.track,
            suffix: t.suffix,
        }
    }
}

fn artist_detail_from_catalog(
    id: &str,
    artists: Vec<Artist>,
    albums: Vec<Album>,
) -> Option<ArtistDetail> {
    let artist = artists.into_iter().find(|artist| artist.id == id)?;
    Some(scoped_artist_detail(artist, albums))
}

fn scoped_artist_detail(mut artist: Artist, albums: Vec<Album>) -> ArtistDetail {
    let has_attribution = albums_have_artist_attribution(&albums);
    let albums = if has_attribution {
        albums
            .into_iter()
            .filter(|album| {
                album.artist_id.as_deref() == Some(artist.id.as_str())
                    || album
                        .artist
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(&artist.name))
            })
            .collect()
    } else {
        albums
    };
    artist.album_count = Some(albums.len() as u32);
    if artist.cover_art.is_none() {
        artist.cover_art = albums.iter().find_map(|album| album.cover_art.clone());
    }
    ArtistDetail { artist, albums }
}

fn albums_have_artist_attribution(albums: &[Album]) -> bool {
    albums.iter().any(|album| {
        album.artist_id.is_some()
            || album
                .artist
                .as_deref()
                .is_some_and(|value| !value.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_auth_does_not_add_a_username() {
        let client = SubsonicClient::new(
            "https://music.example.test",
            Credentials::ApiKey("secret key".into()),
        )
        .unwrap();
        let url = client.endpoint("ping", &[]).unwrap();
        let values: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(values.get("apiKey").map(String::as_str), Some("secret key"));
        assert!(!values.contains_key("u"));
        assert!(!values.contains_key("p"));
    }

    #[test]
    fn password_auth_uses_a_salted_token() {
        let client = SubsonicClient::new(
            "https://music.example.test/base",
            Credentials::Password {
                username: "listener".into(),
                password: "do not transmit".into(),
            },
        )
        .unwrap();
        let url = client
            .endpoint("stream", &[("id", "track/1".into())])
            .unwrap();
        let values: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(values.get("u").map(String::as_str), Some("listener"));
        assert_eq!(values.get("id").map(String::as_str), Some("track/1"));
        assert_eq!(values.get("s").map(String::len), Some(16));
        assert_eq!(values.get("t").map(String::len), Some(32));
        assert!(!values.contains_key("p"));
        assert!(!url.as_str().contains("do%20not%20transmit"));
    }

    #[test]
    fn response_envelopes_deserialize_with_flattened_bodies() {
        let envelope: Value = serde_json::from_str(
            r#"{"subsonic-response":{"status":"ok","version":"1.16.1","albumList2":{"album":[{"id":123,"name":"Signals","songCount":"4","year":-1,"duration":"-1"}]}}}"#,
        ).unwrap();
        let response = envelope.get("subsonic-response").unwrap().clone();
        let metadata: ResponseMetadata = serde_json::from_value(response.clone()).unwrap();
        assert_eq!(metadata.status, "ok");
        let albums: AlbumListBody = serde_json::from_value(response).unwrap();
        assert_eq!(albums.album_list.album[0].id, "123");
        assert_eq!(albums.album_list.album[0].song_count, Some(4));
        assert_eq!(albums.album_list.album[0].year, None);
        assert_eq!(albums.album_list.album[0].duration, None);

        let mut legacy: LegacyAlbumListBody = serde_json::from_str(
            r#"{"albumList":{"album":[{"id":"bc1","title":"Bandcamp Album","artist":"Artist"}]}}"#,
        )
        .unwrap();
        let legacy_album = legacy.album_list.album.remove(0);
        assert_eq!(legacy_album.name, "Bandcamp Album");

        let directory: MusicDirectoryBody = serde_json::from_str(
            r#"{"directory":{"id":456,"name":"Bandcamp Album","child":[{"id":789,"title":"Track One","isDir":false,"duration":"180"}]}}"#,
        ).unwrap();
        assert_eq!(directory.directory.id, "456");
        assert_eq!(directory.directory.child[0].duration, Some(180));

        let artists: ArtistsBody = serde_json::from_str(
            r#"{"artists":{"ignoredArticles":"The El La Los Las Le Les","index":[{"name":"A","artist":[{"id":42,"name":"Artist One","albumCount":"3","coverArt":99}]}]}}"#,
        )
        .unwrap();
        let artist = &artists.artists.index[0].artist[0];
        assert_eq!(artist.id, "42");
        assert_eq!(artist.album_count, Some(3));
        assert_eq!(artist.cover_art.as_deref(), Some("99"));

        let detail: ArtistBody = serde_json::from_str(
            r#"{"artist":{"id":42,"name":"Artist One","albumCount":"2","coverArt":99,"album":[{"id":10,"name":"First","artist":"Artist One","artistId":42},{"id":11,"name":"Second","artist":"Artist One","artistId":42}]}}"#,
        )
        .unwrap();
        assert_eq!(detail.artist.artist.id, "42");
        assert_eq!(detail.artist.album.len(), 2);
        assert_eq!(detail.artist.album[1].name, "Second");

        let playlist: PlaylistBody = serde_json::from_str(
            r#"{"playlist":{"id":7,"name":"Favorites","owner":"listener","songCount":"1","duration":"180","coverArt":88,"entry":[{"id":9,"title":"Track One","duration":"180"}]}}"#,
        )
        .unwrap();
        assert_eq!(playlist.playlist.id, "7");
        assert_eq!(playlist.playlist.song_count, Some(1));
        assert_eq!(playlist.playlist.cover_art.as_deref(), Some("88"));
        assert_eq!(playlist.playlist.entry[0].id, "9");
    }

    #[test]
    fn artist_detail_fallback_matches_releases_by_id_or_name() {
        let artists = vec![Artist {
            id: "name:Artist One".into(),
            name: "Artist One".into(),
            album_count: None,
            cover_art: None,
        }];
        let albums = vec![
            Album {
                id: "one".into(),
                name: "First".into(),
                artist: Some("Artist One".into()),
                artist_id: None,
                cover_art: Some("cover-one".into()),
                song_count: Some(1),
                duration_seconds: None,
                year: None,
            },
            Album {
                id: "other".into(),
                name: "Other".into(),
                artist: Some("Someone Else".into()),
                artist_id: Some("someone-else".into()),
                cover_art: None,
                song_count: Some(1),
                duration_seconds: None,
                year: None,
            },
        ];

        let detail = artist_detail_from_catalog("name:Artist One", artists, albums).unwrap();
        assert_eq!(detail.albums.len(), 1);
        assert_eq!(detail.artist.album_count, Some(1));
        assert_eq!(detail.artist.cover_art.as_deref(), Some("cover-one"));
    }

    #[test]
    fn successful_artist_detail_drops_unrelated_collection_albums() {
        let artist = Artist {
            id: "blanck-mass".into(),
            name: "Blanck Mass".into(),
            album_count: Some(1),
            cover_art: None,
        };
        let albums = vec![
            Album {
                id: "matching-name".into(),
                name: "World Eater".into(),
                artist: Some("Blanck Mass".into()),
                artist_id: None,
                cover_art: None,
                song_count: None,
                duration_seconds: None,
                year: Some(2017),
            },
            Album {
                id: "unrelated".into(),
                name: "Another Purchase".into(),
                artist: Some("Someone Else".into()),
                artist_id: Some("someone-else".into()),
                cover_art: None,
                song_count: None,
                duration_seconds: None,
                year: None,
            },
        ];

        let detail = scoped_artist_detail(artist, albums);
        assert_eq!(detail.albums.len(), 1);
        assert_eq!(detail.albums[0].id, "matching-name");
        assert_eq!(detail.artist.album_count, Some(1));
    }

    #[test]
    fn unattributed_artist_responses_are_detectable_as_unscoped() {
        let albums = vec![Album {
            id: "purchase".into(),
            name: "Unattributed Purchase".into(),
            artist: None,
            artist_id: None,
            cover_art: None,
            song_count: None,
            duration_seconds: None,
            year: None,
        }];

        assert!(!albums_have_artist_attribution(&albums));
    }
}
