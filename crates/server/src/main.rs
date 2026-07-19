use std::{collections::HashMap, env, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get},
};
use resonance_core::{
    AggregateResponse, Album, AlbumDetail, LibraryAlbum, LibraryAlbumDetail, LibraryTrack, MediaId,
    MediaKind, MusicProvider, ProviderError, ProviderId, ProviderIssue, ProviderIssueKind, Track,
};
use resonance_provider_subsonic::{Credentials, SubsonicClient};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    providers: Arc<RwLock<HashMap<String, ProviderEntry>>>,
    http: reqwest::Client,
}

#[derive(Clone)]
struct ProviderEntry {
    summary: ProviderSummary,
    client: Arc<dyn MusicProvider>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderSummary {
    id: String,
    name: String,
    url: String,
    username: String,
    auth: String,
    server_type: String,
    server_version: String,
    open_subsonic: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterProvider {
    name: String,
    url: String,
    username: Option<String>,
    auth: AuthMethod,
    secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum AuthMethod {
    Password,
    ApiKey,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "resonance_server=info,tower_http=info".into()),
        )
        .init();

    let state = AppState {
        providers: Arc::new(RwLock::new(HashMap::new())),
        http: reqwest::Client::new(),
    };
    if let Some(entry) = provider_from_environment().await {
        state
            .providers
            .write()
            .await
            .insert(entry.summary.id.clone(), entry);
    }

    let app = Router::new()
        .route(
            "/api/providers",
            get(list_providers).post(register_provider),
        )
        .route("/api/providers/{provider_id}", delete(remove_provider))
        .route("/api/providers/{provider_id}/status", get(status))
        .route("/api/providers/{provider_id}/albums", get(albums))
        .route("/api/providers/{provider_id}/albums/{album_id}", get(album))
        .route("/api/providers/{provider_id}/search", get(search))
        .route("/api/library/albums", get(library_albums))
        .route(
            "/api/library/albums/{provider_id}/{album_id}",
            get(library_album),
        )
        .route("/api/library/search", get(library_search))
        .route(
            "/api/providers/{provider_id}/tracks/{track_id}/stream",
            get(stream),
        )
        .route("/api/providers/{provider_id}/covers/{cover_id}", get(cover))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);
    let address: SocketAddr = env::var("RESONANCE_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()
        .expect("invalid RESONANCE_BIND");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind server");
    tracing::info!(%address, "Resonance API listening");
    axum::serve(listener, app).await.expect("server failed");
}

async fn provider_from_environment() -> Option<ProviderEntry> {
    let url = env::var("RESONANCE_SERVER_URL").ok()?;
    let (credentials, username, auth) = if let Ok(key) = env::var("RESONANCE_API_KEY") {
        (
            Credentials::ApiKey(key),
            String::new(),
            "API Key".to_string(),
        )
    } else {
        let username = env::var("RESONANCE_USERNAME").ok()?;
        let password = env::var("RESONANCE_PASSWORD").ok()?;
        (
            Credentials::Password {
                username: username.clone(),
                password,
            },
            username,
            "Password".to_string(),
        )
    };
    match build_entry(
        "environment".into(),
        "Default server".into(),
        url,
        username,
        auth,
        credentials,
    )
    .await
    {
        Ok(entry) => Some(entry),
        Err(error) => {
            tracing::warn!(error = %error.0, "environment provider could not be registered");
            None
        }
    }
}

async fn build_entry(
    id: String,
    name: String,
    url: String,
    username: String,
    auth: String,
    credentials: Credentials,
) -> Result<ProviderEntry, ApiError> {
    let client = SubsonicClient::new(&url, credentials)?;
    let status = client.ping().await?;
    Ok(ProviderEntry {
        summary: ProviderSummary {
            id,
            name,
            url,
            username,
            auth,
            server_type: status.provider,
            server_version: status.server_version,
            open_subsonic: status.open_subsonic,
        },
        client: Arc::new(client),
    })
}

async fn list_providers(State(state): State<AppState>) -> Json<Vec<ProviderSummary>> {
    let mut providers: Vec<_> = state
        .providers
        .read()
        .await
        .values()
        .map(|entry| entry.summary.clone())
        .collect();
    providers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Json(providers)
}

async fn register_provider(
    State(state): State<AppState>,
    Json(request): Json<RegisterProvider>,
) -> Result<(StatusCode, Json<ProviderSummary>), ApiError> {
    if request.name.trim().is_empty() || request.url.trim().is_empty() || request.secret.is_empty()
    {
        return Err(ApiError::bad_request(
            "name, URL, and credentials are required",
        ));
    }
    let username = request.username.unwrap_or_default();
    let (auth, credentials) = match request.auth {
        AuthMethod::ApiKey => ("API Key".into(), Credentials::ApiKey(request.secret)),
        AuthMethod::Password => {
            if username.trim().is_empty() {
                return Err(ApiError::bad_request(
                    "username is required for password authentication",
                ));
            }
            (
                "Password".into(),
                Credentials::Password {
                    username: username.clone(),
                    password: request.secret,
                },
            )
        }
    };
    let id = uuid::Uuid::new_v4().to_string();
    let entry = build_entry(
        id.clone(),
        request.name.trim().into(),
        request.url.trim().into(),
        username,
        auth,
        credentials,
    )
    .await?;
    let summary = entry.summary.clone();
    state.providers.write().await.insert(id, entry);
    Ok((StatusCode::CREATED, Json(summary)))
}

async fn remove_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .providers
        .write()
        .await
        .remove(&provider_id)
        .ok_or_else(|| ApiError(ProviderError::NotFound))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn provider(state: &AppState, id: &str) -> Result<Arc<dyn MusicProvider>, ApiError> {
    state
        .providers
        .read()
        .await
        .get(id)
        .map(|entry| entry.client.clone())
        .ok_or_else(|| ApiError(ProviderError::NotFound))
}

async fn status(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<impl Serialize>, ApiError> {
    Ok(Json(provider(&state, &provider_id).await?.ping().await?))
}

#[derive(Deserialize)]
struct PageQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}
async fn albums(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Json<impl Serialize>, ApiError> {
    Ok(Json(
        provider(&state, &provider_id)
            .await?
            .albums(q.limit.unwrap_or(30), q.offset.unwrap_or(0))
            .await?,
    ))
}
async fn album(
    State(state): State<AppState>,
    Path((provider_id, album_id)): Path<(String, String)>,
) -> Result<Json<impl Serialize>, ApiError> {
    Ok(Json(
        provider(&state, &provider_id)
            .await?
            .album(&album_id)
            .await?,
    ))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<u32>,
}
async fn search(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<impl Serialize>, ApiError> {
    Ok(Json(
        provider(&state, &provider_id)
            .await?
            .search(&q.q, q.limit.unwrap_or(50))
            .await?,
    ))
}

const PROVIDER_QUERY_TIMEOUT: Duration = Duration::from_secs(20);

fn qualified_id(provider_id: &str, kind: MediaKind, item_id: String) -> MediaId {
    MediaId::new(ProviderId::new(provider_id), kind, item_id)
}

fn qualify_album(summary: &ProviderSummary, album: Album) -> LibraryAlbum {
    LibraryAlbum {
        id: qualified_id(&summary.id, MediaKind::Album, album.id),
        name: album.name,
        artist: album.artist,
        artist_id: album
            .artist_id
            .map(|id| qualified_id(&summary.id, MediaKind::Artist, id)),
        cover_art: album
            .cover_art
            .map(|id| qualified_id(&summary.id, MediaKind::Artwork, id)),
        song_count: album.song_count,
        duration_seconds: album.duration_seconds,
        year: album.year,
        source_name: summary.name.clone(),
    }
}

fn qualify_track(summary: &ProviderSummary, track: Track) -> LibraryTrack {
    LibraryTrack {
        id: qualified_id(&summary.id, MediaKind::Track, track.id),
        title: track.title,
        artist: track.artist,
        artist_id: track
            .artist_id
            .map(|id| qualified_id(&summary.id, MediaKind::Artist, id)),
        album: track.album,
        album_id: track
            .album_id
            .map(|id| qualified_id(&summary.id, MediaKind::Album, id)),
        cover_art: track
            .cover_art
            .map(|id| qualified_id(&summary.id, MediaKind::Artwork, id)),
        duration_seconds: track.duration_seconds,
        track_number: track.track_number,
        suffix: track.suffix,
        source_name: summary.name.clone(),
    }
}

fn provider_issue(summary: &ProviderSummary, error: ProviderError) -> ProviderIssue {
    let (kind, retryable) = match &error {
        ProviderError::Unauthorized => (ProviderIssueKind::Unauthorized, false),
        ProviderError::Unavailable(_) => (ProviderIssueKind::Unavailable, true),
        ProviderError::InvalidResponse(_) => (ProviderIssueKind::InvalidResponse, false),
        ProviderError::Unsupported(_) => (ProviderIssueKind::Unsupported, false),
        _ => (ProviderIssueKind::Other, false),
    };
    ProviderIssue {
        provider_id: ProviderId::new(summary.id.as_str()),
        provider_name: summary.name.clone(),
        kind,
        message: error.to_string(),
        retryable,
    }
}

fn timeout_issue(summary: &ProviderSummary) -> ProviderIssue {
    ProviderIssue {
        provider_id: ProviderId::new(summary.id.as_str()),
        provider_name: summary.name.clone(),
        kind: ProviderIssueKind::Timeout,
        message: "provider query timed out".into(),
        retryable: true,
    }
}

async fn provider_entries(state: &AppState) -> Vec<ProviderEntry> {
    state.providers.read().await.values().cloned().collect()
}

async fn library_albums(
    State(state): State<AppState>,
    Query(q): Query<PageQuery>,
) -> Json<AggregateResponse<LibraryAlbum>> {
    let limit = q.limit.unwrap_or(30).clamp(1, 500);
    let offset = q.offset.unwrap_or(0);
    let mut tasks = tokio::task::JoinSet::new();
    for entry in provider_entries(&state).await {
        tasks.spawn(async move {
            let result = tokio::time::timeout(
                PROVIDER_QUERY_TIMEOUT,
                entry.client.albums(limit.saturating_add(offset), 0),
            )
            .await;
            (entry.summary, result)
        });
    }

    let mut items = Vec::new();
    let mut issues = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((summary, Ok(Ok(albums)))) => {
                items.extend(
                    albums
                        .into_iter()
                        .map(|album| qualify_album(&summary, album)),
                );
            }
            Ok((summary, Ok(Err(error)))) => issues.push(provider_issue(&summary, error)),
            Ok((summary, Err(_))) => issues.push(timeout_issue(&summary)),
            Err(error) => tracing::warn!(%error, "library album task failed"),
        }
    }
    items.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    items = items
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    issues.sort_by(|a, b| a.provider_id.cmp(&b.provider_id));
    Json(AggregateResponse {
        complete: issues.is_empty(),
        items,
        issues,
    })
}

async fn library_album(
    State(state): State<AppState>,
    Path((provider_id, album_id)): Path<(String, String)>,
) -> Result<Json<LibraryAlbumDetail>, ApiError> {
    let entry = state
        .providers
        .read()
        .await
        .get(&provider_id)
        .cloned()
        .ok_or_else(|| ApiError(ProviderError::NotFound))?;
    let AlbumDetail { album, tracks } = entry.client.album(&album_id).await?;
    Ok(Json(LibraryAlbumDetail {
        album: qualify_album(&entry.summary, album),
        tracks: tracks
            .into_iter()
            .map(|track| qualify_track(&entry.summary, track))
            .collect(),
    }))
}

async fn library_search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Json<AggregateResponse<LibraryTrack>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let mut tasks = tokio::task::JoinSet::new();
    for entry in provider_entries(&state).await {
        let query = q.q.clone();
        tasks.spawn(async move {
            let result =
                tokio::time::timeout(PROVIDER_QUERY_TIMEOUT, entry.client.search(&query, limit))
                    .await;
            (entry.summary, result)
        });
    }

    let mut items = Vec::new();
    let mut issues = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((summary, Ok(Ok(tracks)))) => {
                items.extend(
                    tracks
                        .into_iter()
                        .map(|track| qualify_track(&summary, track)),
                );
            }
            Ok((summary, Ok(Err(error)))) => issues.push(provider_issue(&summary, error)),
            Ok((summary, Err(_))) => issues.push(timeout_issue(&summary)),
            Err(error) => tracing::warn!(%error, "library search task failed"),
        }
    }
    items.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    items.truncate(limit as usize);
    issues.sort_by(|a, b| a.provider_id.cmp(&b.provider_id));
    Json(AggregateResponse {
        complete: issues.is_empty(),
        items,
        issues,
    })
}

async fn stream(
    State(state): State<AppState>,
    Path((provider_id, track_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let url = provider(&state, &provider_id)
        .await?
        .stream_url(&track_id)?;
    proxy(&state.http, url, headers.get(header::RANGE).cloned()).await
}

#[derive(Deserialize)]
struct CoverQuery {
    size: Option<u32>,
}
async fn cover(
    State(state): State<AppState>,
    Path((provider_id, cover_id)): Path<(String, String)>,
    Query(q): Query<CoverQuery>,
) -> Result<Response, ApiError> {
    let url = provider(&state, &provider_id)
        .await?
        .cover_art_url(&cover_id, q.size)?;
    proxy(&state.http, url, None).await
}

async fn proxy(
    http: &reqwest::Client,
    url: String,
    range: Option<header::HeaderValue>,
) -> Result<Response, ApiError> {
    let mut request = http.get(url);
    if let Some(value) = range {
        request = request.header(header::RANGE, value);
    }
    let upstream = request
        .send()
        .await
        .map_err(|e| ApiError(ProviderError::Unavailable(e.to_string())))?;
    let status = upstream.status();
    let response_headers = upstream.headers().clone();
    let mut builder = Response::builder().status(status);
    for name in [
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::ACCEPT_RANGES,
    ] {
        if let Some(value) = response_headers.get(&name) {
            builder = builder.header(name, value);
        }
    }
    builder
        .body(Body::from_stream(upstream.bytes_stream()))
        .map_err(|e| ApiError(ProviderError::Unavailable(e.to_string())))
}

struct ApiError(ProviderError);
impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self(ProviderError::InvalidResponse(message.into()))
    }
}
impl From<ProviderError> for ApiError {
    fn from(value: ProviderError) -> Self {
        Self(value)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            ProviderError::Unauthorized => StatusCode::UNAUTHORIZED,
            ProviderError::NotFound => StatusCode::NOT_FOUND,
            ProviderError::Unavailable(_) => StatusCode::BAD_GATEWAY,
            ProviderError::InvalidResponse(_) => StatusCode::BAD_REQUEST,
            ProviderError::Unsupported(_) => StatusCode::NOT_IMPLEMENTED,
            _ => StatusCode::BAD_GATEWAY,
        };
        (
            status,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}
