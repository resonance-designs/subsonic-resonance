use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

const API_BASE: &str = "http://127.0.0.1:3000/api";
const QUEUE_STORAGE_KEY: &str = "resonance.playbackQueue.v1";
const SCROBBLING_STORAGE_KEY: &str = "resonance.scrobblingEnabled.v1";
const LIBRARY_PAGE_SIZE: usize = 500;
const ALBUMS_PER_PAGE: usize = 24;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Albums,
    Artists,
    Playlists,
    Favorites,
    Search,
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum RepeatMode {
    #[default]
    Off,
    One,
    All,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Provider {
    id: String,
    name: String,
    url: String,
    username: String,
    auth: String,
    server_type: String,
    server_version: String,
    api_version: String,
    open_subsonic: bool,
    favorites_supported: bool,
    scrobbling_supported: bool,
    capabilities_known: bool,
    capabilities: Vec<ProviderCapability>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderCapability {
    name: String,
    versions: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaId {
    provider_id: String,
    #[allow(dead_code)]
    kind: String,
    item_id: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Album {
    id: MediaId,
    name: String,
    artist: Option<String>,
    cover_art: Option<MediaId>,
    song_count: Option<u32>,
    duration_seconds: Option<u64>,
    year: Option<u32>,
    source_name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Artist {
    id: MediaId,
    name: String,
    album_count: Option<u32>,
    cover_art: Option<MediaId>,
    source_name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtistDetailResponse {
    artist: Artist,
    albums: Vec<Album>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Playlist {
    id: MediaId,
    name: String,
    owner: Option<String>,
    song_count: Option<u32>,
    duration_seconds: Option<u64>,
    cover_art: Option<MediaId>,
    source_name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Track {
    id: MediaId,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    cover_art: Option<MediaId>,
    duration_seconds: Option<u64>,
    track_number: Option<u32>,
    source_name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueueSnapshot {
    version: u8,
    tracks: Vec<Track>,
    current: usize,
    #[serde(default)]
    position_seconds: f64,
    #[serde(default = "default_volume")]
    volume: f64,
    #[serde(default)]
    muted: bool,
    #[serde(default)]
    shuffle: bool,
    #[serde(default)]
    repeat: RepeatMode,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumDetail {
    album: Album,
    tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderIssue {
    provider_name: String,
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AggregateResponse<T> {
    items: Vec<T>,
    issues: Vec<ProviderIssue>,
    #[allow(dead_code)]
    complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FavoriteCollection {
    artists: Vec<Artist>,
    albums: Vec<Album>,
    tracks: Vec<Track>,
    issues: Vec<ProviderIssue>,
    #[allow(dead_code)]
    complete: bool,
}

#[derive(Clone)]
enum FavoriteItem {
    Artist(Artist),
    Album(Album),
    Track(Track),
}

impl FavoriteItem {
    fn id(&self) -> &MediaId {
        match self {
            Self::Artist(item) => &item.id,
            Self::Album(item) => &item.id,
            Self::Track(item) => &item.id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterProvider {
    name: String,
    url: String,
    username: Option<String>,
    auth: AuthMethod,
    secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum AuthMethod {
    Password,
    ApiKey,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScrobbleRequest {
    submission: bool,
    time_ms: Option<u64>,
}

fn api(path: &str) -> String {
    format!("{API_BASE}{path}")
}
fn encode(value: &str) -> String {
    js_sys::encode_uri_component(value)
        .as_string()
        .unwrap_or_default()
}

fn load_scrobbling_enabled() -> bool {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(SCROBBLING_STORAGE_KEY).ok().flatten())
        .and_then(|value| value.parse().ok())
        .unwrap_or(true)
}

fn persist_scrobbling_enabled(enabled: bool) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(SCROBBLING_STORAGE_KEY, &enabled.to_string());
    }
}

fn scrobble_threshold(duration_seconds: f64) -> f64 {
    if duration_seconds.is_finite() && duration_seconds > 0.0 {
        (duration_seconds / 2.0).min(240.0)
    } else {
        240.0
    }
}

fn forward_playback_delta(previous: Option<f64>, current: f64) -> f64 {
    previous
        .map(|previous| current - previous)
        .filter(|delta| delta.is_finite() && *delta > 0.0 && *delta <= 5.0)
        .unwrap_or(0.0)
}

fn should_submit_scrobble(listened: f64, duration: f64, already_submitted: bool) -> bool {
    !already_submitted && listened >= scrobble_threshold(duration)
}

async fn report_scrobble(track: MediaId, submission: bool, time_ms: u64) -> Result<(), String> {
    let request = ScrobbleRequest {
        submission,
        time_ms: Some(time_ms),
    };
    let response = Request::post(&api(&format!(
        "/library/scrobble/{}/{}",
        encode(&track.provider_id),
        encode(&track.item_id)
    )))
    .json(&request)
    .map_err(|error| error.to_string())?
    .send()
    .await
    .map_err(|error| format!("Cannot reach Resonance backend: {error}"))?;
    if response.ok() {
        Ok(())
    } else {
        Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Scrobble request failed".into()))
    }
}

fn stream_url(track: &Track) -> String {
    api(&format!(
        "/providers/{}/tracks/{}/stream",
        encode(&track.id.provider_id),
        encode(&track.id.item_id)
    ))
}

fn start_track_playback(track: &Track) {
    let Some(audio) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("resonance-audio"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlAudioElement>().ok())
    else {
        return;
    };
    audio.set_src(&stream_url(track));
    let _ = audio.play();
}

fn default_volume() -> f64 {
    1.0
}

fn normalized_volume(volume: f64) -> f64 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        default_volume()
    }
}

fn normalized_position(position_seconds: f64, duration_seconds: Option<u64>) -> f64 {
    if !position_seconds.is_finite() || position_seconds < 0.0 {
        return 0.0;
    }
    if duration_seconds.is_some_and(|duration| {
        duration > 0 && position_seconds >= duration.saturating_sub(2) as f64
    }) {
        return 0.0;
    }
    position_seconds
}

fn load_queue_snapshot() -> QueueSnapshot {
    let fallback = || QueueSnapshot {
        version: 2,
        tracks: Vec::new(),
        current: 0,
        position_seconds: 0.0,
        volume: default_volume(),
        muted: false,
        shuffle: false,
        repeat: RepeatMode::Off,
    };
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return fallback();
    };
    let Some(serialized) = storage.get_item(QUEUE_STORAGE_KEY).ok().flatten() else {
        return fallback();
    };
    let Ok(mut snapshot) = serde_json::from_str::<QueueSnapshot>(&serialized) else {
        return fallback();
    };
    if !matches!(snapshot.version, 1 | 2) {
        return fallback();
    }
    snapshot.version = 2;
    snapshot.current = snapshot
        .current
        .min(snapshot.tracks.len().saturating_sub(1));
    snapshot.volume = normalized_volume(snapshot.volume);
    snapshot.position_seconds = snapshot
        .tracks
        .get(snapshot.current)
        .map(|track| normalized_position(snapshot.position_seconds, track.duration_seconds))
        .unwrap_or(0.0);
    snapshot
}

fn persist_playback(
    tracks: &[Track],
    current: usize,
    position_seconds: f64,
    volume: f64,
    muted: bool,
    shuffle: bool,
    repeat: RepeatMode,
) {
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return;
    };
    let current = current.min(tracks.len().saturating_sub(1));
    let snapshot = QueueSnapshot {
        version: 2,
        tracks: tracks.to_vec(),
        current,
        position_seconds: tracks
            .get(current)
            .map(|track| normalized_position(position_seconds, track.duration_seconds))
            .unwrap_or(0.0),
        volume: normalized_volume(volume),
        muted,
        shuffle,
        repeat,
    };
    if let Ok(serialized) = serde_json::to_string(&snapshot) {
        let _ = storage.set_item(QUEUE_STORAGE_KEY, &serialized);
    }
}

async fn get_json<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let response = Request::get(&api(path))
        .send()
        .await
        .map_err(|e| format!("Cannot reach Resonance backend: {e}"))?;
    if !response.ok() {
        let status = response.status();
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| format!("Request failed ({status})")));
    }
    response
        .json()
        .await
        .map_err(|e| format!("Invalid backend response: {e}"))
}

async fn load_library(
    albums: RwSignal<Vec<Album>>,
    loading: RwSignal<bool>,
    issues: RwSignal<Vec<ProviderIssue>>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    error.set(None);
    let mut loaded_albums = Vec::new();
    let mut provider_issues = Vec::new();
    let mut offset = 0_usize;
    loop {
        match get_json::<AggregateResponse<Album>>(&format!(
            "/library/albums?limit={LIBRARY_PAGE_SIZE}&offset={offset}"
        ))
        .await
        {
            Ok(response) => {
                let item_count = response.items.len();
                loaded_albums.extend(response.items);
                for issue in response.issues {
                    if !provider_issues.contains(&issue) {
                        provider_issues.push(issue);
                    }
                }
                if item_count < LIBRARY_PAGE_SIZE {
                    albums.set(loaded_albums);
                    issues.set(provider_issues);
                    break;
                }
                offset = offset.saturating_add(item_count);
            }
            Err(message) => {
                albums.set(Vec::new());
                issues.set(Vec::new());
                error.set(Some(message));
                break;
            }
        }
    }
    loading.set(false);
}

fn page_count(item_count: usize, page_size: usize) -> usize {
    item_count.div_ceil(page_size).max(1)
}

fn cover_url(cover: Option<&MediaId>, size: u32) -> Option<String> {
    cover.map(|id| {
        api(&format!(
            "/providers/{}/covers/{}?size={size}",
            encode(&id.provider_id),
            encode(&id.item_id)
        ))
    })
}

#[component]
fn Artwork(
    cover: Option<MediaId>,
    title: String,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let src = cover_url(cover.as_ref(), 500);
    view! { <div class=format!("cover {class}")>{src.map(|src|view!{<img src=src alt=format!("Cover for {title}")/>})}<span>{title}</span></div> }
}

#[component]
fn Nav(active: RwSignal<Page>, providers: RwSignal<Vec<Provider>>) -> impl IntoView {
    let item = move |page, label, glyph| view! {<button class:active=move||active.get()==page on:click=move |_|active.set(page)><span class="nav-icon">{glyph}</span><span>{label}</span></button>};
    view! {
        <aside class="sidebar">
            <div class="brand"><img class="brand-mark" src="/img/logo.png" alt=""/><div><small class="letter-stretch"><span>"S"</span><span>"u"</span><span>"b"</span><span>"s"</span><span>"o"</span><span>"n"</span><span>"i"</span><span>"c"</span></small><strong>"Resonance"</strong></div></div>
            <nav aria-label="Primary navigation">{item(Page::Home,"Home","⌂")}{item(Page::Albums,"Albums","▣")}{item(Page::Artists,"Artists","◉")}{item(Page::Playlists,"Playlists","≡")}{item(Page::Favorites,"Favorites","★")}{item(Page::Search,"Search","⌕")}{item(Page::Settings,"Settings","⚙")}</nav>
            <section class="sources"><p class="eyebrow">"SOURCES · ALL ACTIVE"</p><div class="source-list">
                {move||providers.get().into_iter().map(|p|view!{<div class="source-button"><i class="online"></i><span><b>{p.name}</b><small>{format!("{} · {}",p.server_type,p.server_version)}</small></span></div>}).collect_view()}
                <Show when=move||providers.get().is_empty()><p class="empty-source">"No connected servers"</p></Show>
            </div></section>
        </aside>
        <header class="mobile-head"><div class="brand"><img class="brand-mark" src="/img/logo.png" alt=""/><div><small class="letter-stretch"><span>"S"</span><span>"u"</span><span>"b"</span><span>"s"</span><span>"o"</span><span>"n"</span><span>"i"</span><span>"c"</span></small><strong>"Resonance"</strong></div></div><button aria-label="Settings" on:click=move |_|active.set(Page::Settings)>"⚙"</button></header>
    }
}

#[component]
fn Home(
    providers: RwSignal<Vec<Provider>>,
    albums: RwSignal<Vec<Album>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    loading: RwSignal<bool>,
    issues: RwSignal<Vec<ProviderIssue>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let browse_tracks = RwSignal::new(Vec::<Track>::new());
    let run_search = move |_| {
        let term = query.get();
        if term.trim().is_empty() {
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match get_json::<AggregateResponse<Track>>(&format!(
                "/library/search?q={}&limit=50",
                encode(&term)
            ))
            .await
            {
                Ok(response) => {
                    browse_tracks.set(response.items);
                    issues.set(response.issues)
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    };
    view! {<main class="content">
        <header class="editorial-head"><div><p class="eyebrow">"ALL CONNECTED SOURCES"</p><h1>"Your library"</h1><p>"Albums and tracks from every available provider."</p></div><form class="search" on:submit=move|e|{e.prevent_default();run_search(())}><span>"⌕"</span><input aria-label="Search library" placeholder="Search all sources" on:input=move|e|query.set(event_target_value(&e))/></form></header>
        <Show when=move||!providers.get().is_empty() fallback=||view!{<section class="empty-library"><h2>"Connect a music server"</h2><p>"Open Settings → Connections and add a provider to begin."</p></section>}>
            <Show when=move||loading.get()><p class="library-state">"Loading connected libraries…"</p></Show>
            <Show when=move||error.get().is_some()>{move||error.get().map(|message|view!{<div class="library-error"><b>"Could not load the unified library"</b><p>{message}</p></div>})}</Show>
            <Show when=move||!issues.get().is_empty()>{move||issues.get().into_iter().map(|issue|view!{<div class="library-error partial"><b>{format!("{} is unavailable",issue.provider_name)}</b><p>{issue.message}</p></div>}).collect_view()}</Show>
            <Show when=move||!albums.get().is_empty()>{move||albums.get().first().cloned().map(|album|view!{<section class="feature real-feature"><Artwork cover=album.cover_art.clone() title=album.name.clone() class="feature-art"/><div class="feature-copy"><p class="eyebrow">"FROM THE UNIFIED LIBRARY"</p><h2>{album.name}</h2><p class="feature-artist">{format!("{} · {}",album.artist.unwrap_or_else(||"Unknown artist".into()),album.source_name)}</p><p class="description">{format!("{} tracks",album.song_count.unwrap_or(0))}</p></div></section>})}</Show>
            <section class="recent"><div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Albums"</h2></div><span class="track-count">{move||format!("{} ALBUMS",albums.get().len())}</span></div><div class="album-row real-albums">
                {move||albums.get().into_iter().map(|album|{let id=album.id.clone();view!{<button class="album" on:click=move |_|{let id=id.clone();loading.set(true);spawn_local(async move{match get_json::<AlbumDetail>(&format!("/library/albums/{}/{}",encode(&id.provider_id),encode(&id.item_id))).await{Ok(detail)=>browse_tracks.set(detail.tracks),Err(message)=>error.set(Some(message))}loading.set(false);});}><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{format!("{} · {}",album.artist.unwrap_or_else(||"Unknown artist".into()),album.source_name)}</span></button>}}).collect_view()}
            </div></section>
            <section class="tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED ALBUM / SEARCH"</p><h2>"Tracks"</h2></div><span class="track-count">{move||format!("{} TRACKS",browse_tracks.get().len())}</span></div><TrackResults items=browse_tracks queue current playing empty_message="Select an album or search to browse tracks."/></section>
        </Show>
    </main>}
}

fn format_duration(seconds: Option<u64>) -> String {
    seconds
        .map(|s| format!("{}:{:02}", s / 60, s % 60))
        .unwrap_or_else(|| "—".into())
}

async fn load_album_tracks(
    id: MediaId,
    tracks: RwSignal<Vec<Track>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    error.set(None);
    match get_json::<AlbumDetail>(&format!(
        "/library/albums/{}/{}",
        encode(&id.provider_id),
        encode(&id.item_id)
    ))
    .await
    {
        Ok(detail) => tracks.set(detail.tracks),
        Err(message) => error.set(Some(message)),
    }
    loading.set(false);
}

#[component]
fn ProviderNotices(issues: RwSignal<Vec<ProviderIssue>>) -> impl IntoView {
    view! {
        <Show when=move || !issues.get().is_empty()>
            {move || issues.get().into_iter().map(|issue| view! {
                <div class="library-error partial">
                    <b>{format!("{} is unavailable", issue.provider_name)}</b>
                    <p>{issue.message}</p>
                </div>
            }).collect_view()}
        </Show>
    }
}

#[component]
fn SourceFilter(providers: RwSignal<Vec<Provider>>, selected: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="source-filter"><span>"Source"</span><select on:change=move |event| selected.set(event_target_value(&event))>
            <option value="">"All sources"</option>
            {move || providers.get().into_iter().map(|provider| view! {<option value=provider.id>{provider.name}</option>}).collect_view()}
        </select></label>
    }
}

#[component]
fn FavoriteButton(item: FavoriteItem) -> impl IntoView {
    let favorites = expect_context::<RwSignal<FavoriteCollection>>();
    let error = expect_context::<RwSignal<Option<String>>>();
    let providers = expect_context::<RwSignal<Vec<Provider>>>();
    let id = item.id().clone();
    let provider_id = id.provider_id.clone();
    let item_for_update = item.clone();
    let is_favorite = Memo::new(move |_| match &item {
        FavoriteItem::Artist(_) => favorites.get().artists.iter().any(|value| value.id == id),
        FavoriteItem::Album(_) => favorites.get().albums.iter().any(|value| value.id == id),
        FavoriteItem::Track(_) => favorites.get().tracks.iter().any(|value| value.id == id),
    });
    let supported = Memo::new(move |_| {
        providers
            .get()
            .into_iter()
            .find(|provider| provider.id == provider_id)
            .is_some_and(|provider| provider.favorites_supported)
    });
    view! {<button type="button" class="favorite-button" class:active=move || is_favorite.get() disabled=move || !supported.get() title=move || if supported.get() {""} else {"Favorites are unavailable for this provider"} aria-label=move || if !supported.get() {"Favorites unavailable"} else if is_favorite.get() {"Remove from favorites"} else {"Add to favorites"} on:click=move |_| {
        if !supported.get_untracked() { return; }
        let favorite = !is_favorite.get_untracked();
        let previous = favorites.get_untracked();
        favorites.update(|collection| match &item_for_update {
            FavoriteItem::Artist(item) => { collection.artists.retain(|value| value.id != item.id); if favorite { collection.artists.push(item.clone()); } },
            FavoriteItem::Album(item) => { collection.albums.retain(|value| value.id != item.id); if favorite { collection.albums.push(item.clone()); } },
            FavoriteItem::Track(item) => { collection.tracks.retain(|value| value.id != item.id); if favorite { collection.tracks.push(item.clone()); } },
        });
        let id = item_for_update.id().clone();
        spawn_local(async move {
            let url = api(&format!("/library/favorites/{}/{}/{}", encode(&id.provider_id), encode(&id.kind), encode(&id.item_id)));
            let result = if favorite { Request::put(&url).send().await } else { Request::delete(&url).send().await };
            match result {
                Ok(response) if response.ok() => {}
                Ok(response) => { favorites.set(previous); error.set(Some(response.text().await.unwrap_or_else(|_| "Could not update favorite".into()))); }
                Err(message) => { favorites.set(previous); error.set(Some(format!("Could not update favorite: {message}"))); }
            }
        });
    }>{move || if is_favorite.get() {"★"} else {"☆"}}</button>}
}

#[component]
fn TrackResults(
    #[prop(into)] items: Signal<Vec<Track>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    empty_message: &'static str,
) -> impl IntoView {
    view! {
        <Show when=move || !items.get().is_empty() fallback=move || view! {
            <div class="empty-results"><p>{empty_message}</p></div>
        }>
            <div class="track-list">
                {move || items.get().into_iter().enumerate().map(|(idx, track)| {
                    let result_items = items;
                    let playback_track = track.clone();
                    let favorite_track = track.clone();
                    view! {
                        <div class="track-row"><button class="track-play" on:click=move |_| {
                            queue.set(result_items.get());
                            current.set(idx);
                            playing.set(true);
                            start_track_playback(&playback_track);
                        }>
                            <span class="track-no">{track.track_number.map(|number| format!("{number:02}")).unwrap_or_else(|| "—".into())}</span>
                            <span class="mini-cover"></span>
                            <span class="track-name"><b>{track.title}</b><small>{format!("{} · {}", track.artist.unwrap_or_else(|| "Unknown artist".into()), track.source_name)}</small></span>
                            <span class="track-album">{track.album.unwrap_or_default()}</span>
                            <span class="track-time">{format_duration(track.duration_seconds)}</span>
                        </button><FavoriteButton item=FavoriteItem::Track(favorite_track)/></div>
                    }
                }).collect_view()}
            </div>
        </Show>
    }
}

#[component]
fn AlbumsPage(
    providers: RwSignal<Vec<Provider>>,
    albums: RwSignal<Vec<Album>>,
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    loading: RwSignal<bool>,
    issues: RwSignal<Vec<ProviderIssue>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let filter = RwSignal::new(String::new());
    let sort = RwSignal::new(String::from("title"));
    let source = RwSignal::new(String::new());
    let page = RwSignal::new(0_usize);
    let selected_album = RwSignal::new(None::<Album>);
    let visible_albums = move || {
        let needle = filter.get().trim().to_lowercase();
        let mut items = albums.get();
        if !needle.is_empty() {
            items.retain(|album| {
                album.name.to_lowercase().contains(&needle)
                    || album
                        .artist
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&needle)
                    || album.source_name.to_lowercase().contains(&needle)
            });
        }
        let provider_id = source.get();
        if !provider_id.is_empty() {
            items.retain(|album| album.id.provider_id == provider_id);
        }
        match sort.get().as_str() {
            "artist" => items.sort_by_key(|album| {
                (
                    album.artist.clone().unwrap_or_default().to_lowercase(),
                    album.name.to_lowercase(),
                )
            }),
            "year" => items.sort_by_key(|album| {
                (
                    std::cmp::Reverse(album.year.unwrap_or(0)),
                    album.name.to_lowercase(),
                )
            }),
            "source" => items
                .sort_by_key(|album| (album.source_name.to_lowercase(), album.name.to_lowercase())),
            _ => items.sort_by_key(|album| album.name.to_lowercase()),
        }
        items
    };
    Effect::new(move |_| {
        filter.get();
        sort.get();
        source.get();
        page.set(0);
    });

    view! {
        <Show
            when=move || selected_album.get().is_some()
            fallback=move || view! {<main class="content library-page albums-page">
                <header class="editorial-head"><div><p class="eyebrow">"UNIFIED LIBRARY"</p><h1>"Albums"</h1><p>"Browse releases from every connected source."</p></div></header>
                <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings to browse albums."</p></section>}>
                    <div class="library-toolbar">
                        <label><span>"Filter albums"</span><input type="search" placeholder="Title, artist, or source" on:input=move |event| filter.set(event_target_value(&event))/></label>
                        <SourceFilter providers selected=source/>
                        <label><span>"Sort by"</span><select on:change=move |event| sort.set(event_target_value(&event))><option value="title">"Title"</option><option value="artist">"Artist"</option><option value="year">"Newest year"</option><option value="source">"Source"</option></select></label>
                    </div>
                    <Show when=move || loading.get()><p class="library-state">"Loading albums…"</p></Show>
                    <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Could not load albums"</b><p>{message}</p></div>})}</Show>
                    <ProviderNotices issues/>
                    <div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Releases"</h2></div><span class="track-count">{move || format!("{} ALBUMS", visible_albums().len())}</span></div>
                    <Show when=move || !visible_albums().is_empty() fallback=move || view! {<div class="empty-results"><p>"No albums match this filter."</p></div>}>
                        <div class="album-grid">{move || visible_albums().into_iter().skip(page.get() * ALBUMS_PER_PAGE).take(ALBUMS_PER_PAGE).map(|album| {
                            let selected = album.clone();
                            view! {<button class="media-card album-card" on:click=move |_| selected_album.set(Some(selected.clone()))><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(|| "Unknown artist".into())}</span><small>{format!("{}{}", album.source_name, album.year.map(|year| format!(" · {year}")).unwrap_or_default())}</small></button>}
                        }).collect_view()}</div>
                        <nav class="library-pagination" aria-label="Album pages">
                            <button type="button" disabled=move || page.get() == 0 on:click=move |_| page.update(|value| *value = value.saturating_sub(1))>"← Previous"</button>
                            <span>{move || format!("Page {} of {}", page.get() + 1, page_count(visible_albums().len(), ALBUMS_PER_PAGE))}</span>
                            <button type="button" disabled=move || page.get() + 1 >= page_count(visible_albums().len(), ALBUMS_PER_PAGE) on:click=move |_| page.update(|value| *value += 1)>"Next →"</button>
                        </nav>
                    </Show>
                </Show>
            </main>}
        >
            {move || selected_album.get().map(|album| view! {<AlbumDetailPage album selected_album queue=tracks current playing/>})}
        </Show>
    }
}

#[component]
fn AlbumDetailPage(
    album: Album,
    selected_album: RwSignal<Option<Album>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let favorite_album = album.clone();
    let id = album.id.clone();
    let title = album.name.clone();
    let artist = album
        .artist
        .clone()
        .unwrap_or_else(|| "Unknown artist".into());
    let source = album.source_name.clone();
    let cover = album.cover_art.clone();
    let year = album.year;
    let song_count = album.song_count;
    let duration = album.duration_seconds;
    let detail = RwSignal::new(None::<AlbumDetail>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    spawn_local(async move {
        match get_json::<AlbumDetail>(&format!(
            "/library/albums/{}/{}",
            encode(&id.provider_id),
            encode(&id.item_id)
        ))
        .await
        {
            Ok(album_detail) => detail.set(Some(album_detail)),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });

    view! {<main class="content library-page album-detail-page">
        <button type="button" class="detail-back" on:click=move |_| selected_album.set(None)>"← All albums"</button>
        <header class="album-detail-head">
            <Artwork cover title=title.clone() class="album-detail-art"/>
            <div><p class="eyebrow">"ALBUM · UNIFIED LIBRARY"</p><h1>{title}</h1><p class="album-detail-artist">{artist}</p><p class="album-detail-meta">{format!("{}{}{}{}", source, year.map(|value| format!(" · {value}")).unwrap_or_default(), song_count.map(|value| format!(" · {value} tracks")).unwrap_or_default(), duration.map(|value| format!(" · {}", format_duration(Some(value)))).unwrap_or_default())}</p><FavoriteButton item=FavoriteItem::Album(favorite_album)/></div>
        </header>
        <Show when=move || loading.get()><p class="library-state">"Loading album…"</p></Show>
        <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Could not load this album"</b><p>{message}</p></div>})}</Show>
        <Show when=move || detail.get().is_some()>{move || detail.get().map(|album_detail| {
            let track_count = album_detail.tracks.len();
            let album_name = album_detail.album.name;
            let album_tracks = RwSignal::new(album_detail.tracks);
            view! {<section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"TRACK LIST"</p><h2>{album_name}</h2></div><span class="track-count">{format!("{track_count} TRACKS")}</span></div><TrackResults items=album_tracks queue current playing empty_message="This album did not return any tracks."/></section>}
        })}</Show>
    </main>}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistDetail {
    #[allow(dead_code)]
    playlist: Playlist,
    tracks: Vec<Track>,
}

#[component]
fn ArtistsPage(
    providers: RwSignal<Vec<Provider>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let artists = RwSignal::new(Vec::<Artist>::new());
    let issues = RwSignal::new(Vec::<ProviderIssue>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let filter = RwSignal::new(String::new());
    let sort = RwSignal::new(String::from("name"));
    let source = RwSignal::new(String::new());
    let request_generation = RwSignal::new(0_u64);
    let selected_artist = RwSignal::new(None::<Artist>);

    Effect::new(move |_| {
        let generation = request_generation.get_untracked().wrapping_add(1);
        request_generation.set(generation);
        if providers.get().is_empty() {
            artists.set(Vec::new());
            issues.set(Vec::new());
            error.set(None);
            loading.set(false);
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let result = get_json::<AggregateResponse<Artist>>("/library/artists?limit=500").await;
            if request_generation.get_untracked() != generation {
                return;
            }
            match result {
                Ok(response) => {
                    artists.set(response.items);
                    issues.set(response.issues);
                }
                Err(message) => {
                    artists.set(Vec::new());
                    issues.set(Vec::new());
                    error.set(Some(message));
                }
            }
            loading.set(false);
        });
    });

    let visible_artists = move || {
        let needle = filter.get().trim().to_lowercase();
        let mut items = artists.get();
        if !needle.is_empty() {
            items.retain(|artist| {
                artist.name.to_lowercase().contains(&needle)
                    || artist.source_name.to_lowercase().contains(&needle)
            });
        }
        let provider_id = source.get();
        if !provider_id.is_empty() {
            items.retain(|artist| artist.id.provider_id == provider_id);
        }
        if sort.get() == "albums" {
            items.sort_by_key(|artist| {
                (
                    std::cmp::Reverse(artist.album_count.unwrap_or(0)),
                    artist.name.to_lowercase(),
                )
            });
        } else {
            items.sort_by_key(|artist| artist.name.to_lowercase());
        }
        items
    };

    view! {
        <Show
            when=move || selected_artist.get().is_some()
            fallback=move || view! {<main class="content library-page artists-page">
                <header class="editorial-head"><div><p class="eyebrow">"UNIFIED LIBRARY"</p><h1>"Artists"</h1><p>"Browse artists from every connected source."</p></div></header>
                <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings to browse artists."</p></section>}>
                    <div class="library-toolbar">
                        <label><span>"Filter artists"</span><input type="search" placeholder="Artist or source" on:input=move |event| filter.set(event_target_value(&event))/></label>
                        <SourceFilter providers selected=source/>
                        <label><span>"Sort by"</span><select on:change=move |event| sort.set(event_target_value(&event))><option value="name">"Name"</option><option value="albums">"Album count"</option></select></label>
                    </div>
                    <Show when=move || loading.get()><p class="library-state">"Loading artists…"</p></Show>
                    <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Could not load artists"</b><p>{message}</p></div>})}</Show>
                    <ProviderNotices issues/>
                    <div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Artists"</h2></div><span class="track-count">{move || format!("{} SHOWN", visible_artists().len())}</span></div>
                    <Show when=move || !visible_artists().is_empty() fallback=move || view! {<div class="empty-results"><p>"No artists match this filter."</p></div>}>
                        <div class="artist-grid">{move || visible_artists().into_iter().map(|artist| {
                            let selected = artist.clone();
                            view! {<button class="media-card artist-card" on:click=move |_| selected_artist.set(Some(selected.clone()))><Artwork cover=artist.cover_art title=artist.name.clone()/><strong>{artist.name}</strong><span>{artist.album_count.map(|count| format!("{count} albums")).unwrap_or_else(|| "Album count unavailable".into())}</span><small>{artist.source_name}</small></button>}
                        }).collect_view()}</div>
                    </Show>
                </Show>
            </main>}
        >
            {move || selected_artist.get().map(|artist| view! {<ArtistDetailPage artist selected_artist queue current playing/>})}
        </Show>
    }
}

#[component]
fn ArtistDetailPage(
    artist: Artist,
    selected_artist: RwSignal<Option<Artist>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let favorite_artist = artist.clone();
    let artist_id = artist.id.clone();
    let artist_name = artist.name.clone();
    let artist_source = artist.source_name.clone();
    let artist_cover = artist.cover_art.clone();
    let releases = RwSignal::new(Vec::<Album>::new());
    let album_tracks = RwSignal::new(Vec::<Track>::new());
    let selected_album = RwSignal::new(None::<MediaId>);
    let loading = RwSignal::new(true);
    let album_loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    spawn_local(async move {
        match get_json::<ArtistDetailResponse>(&format!(
            "/library/artists/{}/{}",
            encode(&artist_id.provider_id),
            encode(&artist_id.item_id)
        ))
        .await
        {
            Ok(mut detail) => {
                detail.albums.sort_by_key(|album| {
                    (
                        std::cmp::Reverse(album.year.unwrap_or(0)),
                        album.name.to_lowercase(),
                    )
                });
                releases.set(detail.albums);
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });

    view! {<main class="content library-page artist-detail-page">
        <button type="button" class="detail-back" on:click=move |_| selected_artist.set(None)>"← All artists"</button>
        <header class="artist-detail-head">
            <Artwork cover=artist_cover title=artist_name.clone() class="artist-detail-art"/>
            <div><p class="eyebrow">"ARTIST · UNIFIED LIBRARY"</p><h1>{artist_name}</h1><p>{artist_source}</p><FavoriteButton item=FavoriteItem::Artist(favorite_artist)/></div>
        </header>
        <Show when=move || loading.get()><p class="library-state">"Loading artist releases…"</p></Show>
        <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Could not load this artist"</b><p>{message}</p></div>})}</Show>
        <Show when=move || !loading.get() && error.get().is_none()>
            <div class="section-title"><div><p class="eyebrow">"DISCOGRAPHY"</p><h2>"Releases"</h2></div><span class="track-count">{move || format!("{} RELEASES", releases.get().len())}</span></div>
            <Show when=move || !releases.get().is_empty() fallback=move || view! {<div class="empty-results"><p>"This provider did not return any releases for the artist."</p></div>}>
                <div class="album-grid">{move || releases.get().into_iter().map(|album| {
                    let id = album.id.clone();
                    let selected_id = id.clone();
                    view! {<button class="media-card album-card" class:selected=move || selected_album.get().as_ref() == Some(&selected_id) disabled=move || album_loading.get() on:click=move |_| {
                        if album_loading.get_untracked() { return; }
                        selected_album.set(Some(id.clone()));
                        spawn_local(load_album_tracks(id.clone(), album_tracks, album_loading, error));
                    }><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(|| "Unknown artist".into())}</span><small>{format!("{}{}", album.source_name, album.year.map(|year| format!(" · {year}")).unwrap_or_default())}</small></button>}
                }).collect_view()}</div>
            </Show>
            <section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED RELEASE"</p><h2>"Tracks"</h2></div><span class="track-count">{move || if selected_album.get().is_some() { format!("{} TRACKS", album_tracks.get().len()) } else { "0 TRACKS".into() }}</span></div><Show when=move || selected_album.get().is_some() fallback=move || view! {<div class="empty-results"><p>"Select a release to view its tracks."</p></div>}><TrackResults items=album_tracks queue current playing empty_message="This release did not return any tracks."/></Show></section>
        </Show>
    </main>}
}

#[component]
fn PlaylistsPage(
    providers: RwSignal<Vec<Provider>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let playlists = RwSignal::new(Vec::<Playlist>::new());
    let playlist_tracks = RwSignal::new(Vec::<Track>::new());
    let issues = RwSignal::new(Vec::<ProviderIssue>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let selected = RwSignal::new(None::<MediaId>);
    let filter = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());
    let request_generation = RwSignal::new(0_u64);

    Effect::new(move |_| {
        let generation = request_generation.get_untracked().wrapping_add(1);
        request_generation.set(generation);
        if providers.get().is_empty() {
            playlists.set(Vec::new());
            playlist_tracks.set(Vec::new());
            issues.set(Vec::new());
            error.set(None);
            loading.set(false);
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let result = get_json::<AggregateResponse<Playlist>>("/library/playlists").await;
            if request_generation.get_untracked() != generation {
                return;
            }
            match result {
                Ok(response) => {
                    playlists.set(response.items);
                    issues.set(response.issues);
                }
                Err(message) => {
                    playlists.set(Vec::new());
                    issues.set(Vec::new());
                    error.set(Some(message));
                }
            }
            loading.set(false);
        });
    });

    let visible_playlists = move || {
        let needle = filter.get().trim().to_lowercase();
        let mut items = playlists.get();
        if !needle.is_empty() {
            items.retain(|playlist| {
                playlist.name.to_lowercase().contains(&needle)
                    || playlist
                        .owner
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&needle)
                    || playlist.source_name.to_lowercase().contains(&needle)
            });
        }
        let provider_id = source.get();
        if !provider_id.is_empty() {
            items.retain(|playlist| playlist.id.provider_id == provider_id);
        }
        items.sort_by_key(|playlist| playlist.name.to_lowercase());
        items
    };

    view! {<main class="content library-page playlists-page">
        <header class="editorial-head"><div><p class="eyebrow">"UNIFIED LIBRARY"</p><h1>"Playlists"</h1><p>"Browse server playlists without changing the active source."</p></div></header>
        <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings to browse playlists."</p></section>}>
            <div class="library-toolbar playlist-toolbar"><label><span>"Filter playlists"</span><input type="search" placeholder="Name, owner, or source" on:input=move |event| filter.set(event_target_value(&event))/></label><SourceFilter providers selected=source/></div>
            <Show when=move || loading.get()><p class="library-state">"Loading playlists…"</p></Show>
            <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Could not load playlists"</b><p>{message}</p></div>})}</Show>
            <ProviderNotices issues/>
            <div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Playlists"</h2></div><span class="track-count">{move || format!("{} SHOWN", visible_playlists().len())}</span></div>
            <Show when=move || !visible_playlists().is_empty() fallback=move || view! {<div class="empty-results"><p>"No playlists match this filter."</p></div>}>
                <div class="playlist-grid">{move || visible_playlists().into_iter().map(|playlist| {
                    let id = playlist.id.clone();
                    let selected_id = id.clone();
                    let duration = format_duration(playlist.duration_seconds);
                    view! {<button class="media-card playlist-card" class:selected=move || selected.get().as_ref() == Some(&selected_id) disabled=move || loading.get() on:click=move |_| {
                        if loading.get_untracked() { return; }
                        loading.set(true);
                        error.set(None);
                        selected.set(Some(id.clone()));
                        let request_id = id.clone();
                        spawn_local(async move {
                            match get_json::<PlaylistDetail>(&format!("/library/playlists/{}/{}", encode(&request_id.provider_id), encode(&request_id.item_id))).await {
                                Ok(detail) => playlist_tracks.set(detail.tracks),
                                Err(message) => error.set(Some(message)),
                            }
                            loading.set(false);
                        });
                    }><Artwork cover=playlist.cover_art title=playlist.name.clone()/><strong>{playlist.name}</strong><span>{playlist.owner.unwrap_or_else(|| "Unknown owner".into())}</span><small>{format!("{} · {} tracks · {duration}", playlist.source_name, playlist.song_count.unwrap_or(0))}</small></button>}
                }).collect_view()}</div>
            </Show>
            <section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED PLAYLIST"</p><h2>"Tracks"</h2></div><span class="track-count">{move || format!("{} TRACKS", playlist_tracks.get().len())}</span></div><Show when=move || selected.get().is_some() fallback=move || view! {<div class="empty-results"><p>"Select a playlist to view its tracks."</p></div>}><TrackResults items=playlist_tracks queue current playing empty_message="This playlist did not return any tracks."/></Show></section>
        </Show>
    </main>}
}

#[component]
fn SearchPage(
    providers: RwSignal<Vec<Provider>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<Track>::new());
    let issues = RwSignal::new(Vec::<ProviderIssue>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let searched = RwSignal::new(false);
    let submitted_query = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());
    let visible_results = Signal::derive(move || {
        let provider_id = source.get();
        results
            .get()
            .into_iter()
            .filter(|track| provider_id.is_empty() || track.id.provider_id == provider_id)
            .collect::<Vec<_>>()
    });
    let submit = move |_| {
        if loading.get_untracked() {
            return;
        }
        let term = query.get().trim().to_string();
        if term.is_empty() {
            return;
        }
        loading.set(true);
        searched.set(true);
        submitted_query.set(term.clone());
        error.set(None);
        spawn_local(async move {
            match get_json::<AggregateResponse<Track>>(&format!(
                "/library/search?q={}&limit=100",
                encode(&term)
            ))
            .await
            {
                Ok(response) => {
                    results.set(response.items);
                    issues.set(response.issues);
                }
                Err(message) => {
                    results.set(Vec::new());
                    issues.set(Vec::new());
                    error.set(Some(message));
                }
            }
            loading.set(false);
        });
    };

    view! {<main class="content library-page search-page">
        <header class="editorial-head"><div><p class="eyebrow">"ALL CONNECTED SOURCES"</p><h1>"Search"</h1><p>"Find tracks across the complete unified library."</p></div></header>
        <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings before searching."</p></section>}>
            <div class="search-toolbar"><form class="library-search" on:submit=move |event| {event.prevent_default(); submit(())}><span>"⌕"</span><input autofocus type="search" aria-label="Search all connected sources" placeholder="Track, artist, or album" on:input=move |event| query.set(event_target_value(&event))/><button class="primary" type="submit" disabled=move || loading.get()>{move || if loading.get() {"Searching…"} else {"Search"}}</button></form><SourceFilter providers selected=source/></div>
            <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Search failed"</b><p>{message}</p></div>})}</Show>
            <ProviderNotices issues/>
            <Show when=move || searched.get()>
                <section class="tracks search-results"><div class="section-title"><div><p class="eyebrow">"SEARCH RESULTS"</p><h2>{move || format!("Results for “{}”", submitted_query.get())}</h2></div><span class="track-count">{move || format!("{} TRACKS", visible_results.get().len())}</span></div><TrackResults items=visible_results queue current playing empty_message="No tracks matched this search."/></section>
            </Show>
            <Show when=move || !searched.get()><div class="search-prompt"><span>"⌕"</span><h2>"Search every source at once"</h2><p>"Results retain their provider identity for reliable playback."</p></div></Show>
        </Show>
    </main>}
}

#[component]
fn FavoritesPage(
    providers: RwSignal<Vec<Provider>>,
    queue: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let favorites = expect_context::<RwSignal<FavoriteCollection>>();
    view! {<main class="content library-page favorites-page">
        <header class="editorial-head"><div><p class="eyebrow">"ALL CONNECTED SOURCES"</p><h1>"Favorites"</h1><p>"Starred artists, albums, and tracks from every available provider."</p></div></header>
        <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings to browse favorites."</p></section>}>
            <Show when=move || providers.get().iter().any(|provider| !provider.favorites_supported)><div class="library-error partial"><b>"Favorites are unavailable on some sources"</b><p>{move || providers.get().into_iter().filter(|provider| !provider.favorites_supported).map(|provider| provider.name).collect::<Vec<_>>().join(", ")}</p></div></Show>
            <Show when=move || !favorites.get().issues.is_empty()>{move || favorites.get().issues.into_iter().map(|issue| view! {<div class="library-error partial"><b>{format!("{} favorites are unavailable", issue.provider_name)}</b><p>{issue.message}</p></div>}).collect_view()}</Show>
            <div class="section-title"><div><p class="eyebrow">"STARRED ARTISTS"</p><h2>"Artists"</h2></div><span class="track-count">{move || format!("{} ARTISTS", favorites.get().artists.len())}</span></div>
            <Show when=move || !favorites.get().artists.is_empty() fallback=move || view! {<div class="empty-results"><p>"No favorite artists yet."</p></div>}><div class="artist-grid">{move || favorites.get().artists.into_iter().map(|artist| {let favorite=artist.clone();view!{<article class="media-card artist-card"><Artwork cover=artist.cover_art title=artist.name.clone()/><strong>{artist.name}</strong><small>{artist.source_name}</small><FavoriteButton item=FavoriteItem::Artist(favorite)/></article>}}).collect_view()}</div></Show>
            <div class="section-title"><div><p class="eyebrow">"STARRED RELEASES"</p><h2>"Albums"</h2></div><span class="track-count">{move || format!("{} ALBUMS", favorites.get().albums.len())}</span></div>
            <Show when=move || !favorites.get().albums.is_empty() fallback=move || view! {<div class="empty-results"><p>"No favorite albums yet."</p></div>}><div class="album-grid">{move || favorites.get().albums.into_iter().map(|album| {let favorite=album.clone();view!{<article class="media-card album-card"><Artwork cover=album.cover_art title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(||"Unknown artist".into())}</span><small>{album.source_name}</small><FavoriteButton item=FavoriteItem::Album(favorite)/></article>}}).collect_view()}</div></Show>
            <section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"STARRED TRACKS"</p><h2>"Tracks"</h2></div><span class="track-count">{move || format!("{} TRACKS", favorites.get().tracks.len())}</span></div><TrackResults items=Signal::derive(move || favorites.get().tracks) queue current playing empty_message="No favorite tracks yet."/></section>
        </Show>
    </main>}
}

#[component]
fn Settings(providers: RwSignal<Vec<Provider>>) -> impl IntoView {
    let scrobbling = expect_context::<RwSignal<bool>>();
    let connections = RwSignal::new(true);
    let dialog = RwSignal::new(false);
    let action_error = RwSignal::new(None::<String>);
    view! {<main class="content settings"><header class="editorial-head"><div><p class="eyebrow">"SYSTEM & SOURCES"</p><h1>"Settings"</h1><p>"Every connection is available throughout Resonance."</p></div></header><div class="settings-tabs" role="tablist"><button role="tab" aria-selected=move||(!connections.get()).to_string() class:active=move||!connections.get() on:click=move |_|connections.set(false)>"General"</button><button role="tab" aria-selected=move||connections.get().to_string() class:active=move||connections.get() on:click=move |_|connections.set(true)>"Connections"</button></div>
        <Show when=move||connections.get() fallback=move ||view!{<section class="general"><div class="preference"><div><b>"Streaming quality"</b><small>"Server transcoding preferences are coming next."</small></div><select><option>"Original"</option></select></div><div class="preference"><div><b>"Scrobbling"</b><small>"Report now-playing and completed tracks to compatible providers."</small></div><label class="toggle-preference"><input type="checkbox" prop:checked=move || scrobbling.get() on:change=move |event| {let enabled=event_target_checked(&event);scrobbling.set(enabled);persist_scrobbling_enabled(enabled);}/><span>{move || if scrobbling.get() {"Enabled"} else {"Disabled"}}</span></label></div></section>}>
            <section class="connections"><div class="connections-head"><div><p class="eyebrow">"LIVE PROVIDERS"</p><h2>"Connections"</h2><p>"All connected providers participate in the unified library."</p></div><button class="primary" on:click=move |_|dialog.set(true)>"＋ Add Provider"</button></div>
            <Show when=move||action_error.get().is_some()>{move||action_error.get().map(|e|view!{<div class="library-error"><p>{e}</p></div>})}</Show>
            <div class="provider-list">{move||providers.get().into_iter().map(|p|{
                let remove_id=p.id.clone();
                let capabilities_known=p.capabilities_known;
                let capability_count=p.capabilities.len();
                let capabilities=p.capabilities.clone();
                let capability_state=if capabilities_known {format!("{capability_count} OpenSubsonic extensions detected")} else if p.open_subsonic {"Extension discovery unavailable; optional features remain disabled".into()} else {"Legacy Subsonic compatibility; optional features remain disabled".into()};
                let annotation_state=format!("Favorites: {} · Scrobbling: {}",if p.favorites_supported {"available"} else {"unavailable"},if p.scrobbling_supported {"available"} else {"unavailable"});
                view!{<article class="active"><span class="signal"><i></i><i></i><i></i></span><div class="provider-identity"><b>{p.name}</b><small>{p.url}</small></div><dl><div><dt>"AUTH"</dt><dd>{p.auth}</dd></div><div><dt>"SERVER"</dt><dd class="connected">{format!("{} {}",p.server_type,p.server_version)}</dd></div><div><dt>"API"</dt><dd>{p.api_version}</dd></div></dl><span class="connected provider-availability">"Available everywhere"</span><button class="remove" on:click=move |_|{let id=remove_id.clone();spawn_local(async move{match Request::delete(&api(&format!("/providers/{}",encode(&id)))).send().await{Ok(response) if response.ok()=>providers.update(|items|items.retain(|x|x.id!=id)),Ok(response)=>action_error.set(Some(response.text().await.unwrap_or_else(|_|"Could not remove provider".into()))),Err(e)=>action_error.set(Some(e.to_string()))}});}>"Remove"</button><div class="provider-capabilities"><span class:known=p.favorites_supported||p.scrobbling_supported>{annotation_state}</span><span class:known=capabilities_known>{capability_state}</span><Show when=move||capabilities_known><ul aria-label="Supported OpenSubsonic extensions">{capabilities.clone().into_iter().map(|capability|view!{<li title=format!("Supported versions: {}",capability.versions.iter().map(u32::to_string).collect::<Vec<_>>().join(", "))>{capability.name}</li>}).collect_view()}</ul></Show></div></article>}
            }).collect_view()}</div>
            <Show when=move||providers.get().is_empty()><div class="empty-library"><h2>"No providers connected"</h2><p>"Add your first Subsonic server above."</p></div></Show>
            <p class="prototype-note">"Web credentials are held only in backend memory and disappear when the backend stops."</p></section>
        </Show><ProviderDialog open=dialog providers error=action_error/>
    </main>}
}

#[component]
fn ProviderDialog(
    open: RwSignal<bool>,
    providers: RwSignal<Vec<Provider>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let api_key = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    Effect::new(move |_| {
        if open.get() {
            if let Some(d) = dialog_ref.get_untracked() {
                let _ = d.show_modal();
            }
        }
    });
    view! {<Show when=move||open.get()><dialog node_ref=dialog_ref class="dialog provider-dialog" on:cancel=move|_:leptos::ev::Event|open.set(false)><button class="dialog-close" on:click=move |_|open.set(false)>"×"</button><p class="eyebrow">"NEW CONNECTION"</p><h2>"Add Provider"</h2><p>"Resonance will verify these details before saving the connection."</p><form on:submit=move|ev|{ev.prevent_default();saving.set(true);error.set(None);let request=RegisterProvider{name:name.get(),url:url.get(),username:Some(username.get()),auth:if api_key.get(){AuthMethod::ApiKey}else{AuthMethod::Password},secret:secret.get()};spawn_local(async move{let builder=match Request::post(&api("/providers")).json(&request){Ok(v)=>v,Err(e)=>{error.set(Some(e.to_string()));saving.set(false);return}};match builder.send().await{Ok(response) if response.ok()=>match response.json::<Provider>().await{Ok(provider)=>{providers.update(|items|items.push(provider));secret.set(String::new());open.set(false)},Err(e)=>error.set(Some(e.to_string()))},Ok(response)=>error.set(Some(response.text().await.unwrap_or_else(|_|"Connection failed".into()))),Err(e)=>error.set(Some(format!("Cannot reach backend: {e}")))}saving.set(false);});}><label>"Name"<input autofocus required placeholder="Home library" on:input=move|e|name.set(event_target_value(&e))/></label><label>"Server URL"<input required type="url" placeholder="https://music.example.com" on:input=move|e|url.set(event_target_value(&e))/></label><label>"Username"<input required=!api_key.get() autocomplete="username" on:input=move|e|username.set(event_target_value(&e))/></label><div class="auth-toggle"><button type="button" class:active=move||!api_key.get() on:click=move |_|api_key.set(false)>"Password"</button><button type="button" class:active=move||api_key.get() on:click=move |_|api_key.set(true)>"API Key"</button></div><label>{move||if api_key.get(){"API Key"}else{"Password"}}<input required type="password" autocomplete="new-password" on:input=move|e|secret.set(event_target_value(&e))/></label><p class="security">"Credentials are sent to the local Rust backend and are not stored in browser storage."</p><div class="dialog-actions"><button type="button" on:click=move |_|open.set(false)>"Cancel"</button><button class="primary" type="submit" disabled=move||saving.get()>{move||if saving.get(){"Connecting…"}else{"Save Provider"}}</button></div></form></dialog></Show>}
}

fn next_queue_index(
    len: usize,
    index: usize,
    shuffle: bool,
    repeat: RepeatMode,
    random: f64,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    if shuffle && len > 1 {
        if repeat == RepeatMode::All {
            let offset = (random * (len - 1) as f64).floor() as usize + 1;
            return Some((index + offset) % len);
        }
        if index + 1 < len {
            let remaining = len - index - 1;
            let offset = (random * remaining as f64).floor() as usize + 1;
            return Some(index + offset);
        }
        return None;
    }
    if index + 1 < len {
        Some(index + 1)
    } else if repeat == RepeatMode::All {
        Some(0)
    } else {
        None
    }
}

fn player_time(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "0:00".into();
    }
    let seconds = seconds.floor() as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn shuffle_icon() -> impl IntoView {
    view! {
        <svg class="player-mode-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
            <path d="M16 3h5v5M4 20l5-5M15 15l6 6M21 16v5h-5M4 4l5 5M15 9l6-6"/>
        </svg>
    }
}

fn repeat_icon(mode: RepeatMode) -> impl IntoView {
    view! {
        <svg class="player-mode-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
            <path d="m17 2 4 4-4 4M3 11V9a3 3 0 0 1 3-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 0 1-3 3H3"/>
            <Show when=move || mode == RepeatMode::One>
                <text class="repeat-one" x="12" y="15">"1"</text>
            </Show>
        </svg>
    }
}

fn volume_icon(is_muted: bool) -> impl IntoView {
    view! {
        <svg class="player-mode-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
            <path d="M11 5 6 9H2v6h4l5 4V5Z"/>
            <Show
                when=move || !is_muted
                fallback=|| view! {<path d="m16 9 5 5M21 9l-5 5"/>}
            >
                <path d="M15 9.5a4 4 0 0 1 0 5M18 7a7 7 0 0 1 0 10"/>
            </Show>
        </svg>
    }
}

#[component]
fn Player(
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    position: RwSignal<f64>,
    volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    shuffle: RwSignal<bool>,
    repeat: RwSignal<RepeatMode>,
) -> impl IntoView {
    let providers = expect_context::<RwSignal<Vec<Provider>>>();
    let scrobbling = expect_context::<RwSignal<bool>>();
    let scrobble_error = expect_context::<RwSignal<Option<String>>>();
    let track = move || tracks.get().get(current.get()).cloned();
    let queue_open = RwSignal::new(false);
    let duration = RwSignal::new(0.0_f64);
    let persisted_second = RwSignal::new(position.get_untracked().floor().max(0.0) as u64);
    let scrobble_track = RwSignal::new(None::<MediaId>);
    let scrobble_started_ms = RwSignal::new(js_sys::Date::now().max(0.0) as u64);
    let now_playing_sent = RwSignal::new(false);
    let submission_sent = RwSignal::new(false);
    let listened_seconds = RwSignal::new(0.0_f64);
    let last_media_position = RwSignal::new(None::<f64>);
    let audio_ref = NodeRef::<leptos::html::Audio>::new();

    Effect::new(move |_| {
        let should_play = playing.get();
        let desired_source = tracks
            .get()
            .get(current.get())
            .map(stream_url)
            .unwrap_or_default();
        if let Some(audio) = audio_ref.get() {
            if audio.get_attribute("src").unwrap_or_default() != desired_source {
                if desired_source.is_empty() {
                    let _ = audio.remove_attribute("src");
                } else {
                    audio.set_src(&desired_source);
                }
                audio.load();
                duration.set(0.0);
            }
            if should_play {
                let _ = audio.play();
            } else {
                audio.pause().ok();
            }
        }
    });

    Effect::new(move |_| {
        let selected = tracks
            .get()
            .get(current.get())
            .map(|track| track.id.clone());
        if scrobble_track.get_untracked() != selected {
            scrobble_track.set(selected);
            scrobble_started_ms.set(js_sys::Date::now().max(0.0) as u64);
            now_playing_sent.set(false);
            submission_sent.set(false);
            listened_seconds.set(0.0);
            last_media_position.set(None);
        }
    });

    let previous = move |_| {
        let len = tracks.get_untracked().len();
        let index = current.get_untracked();
        if index > 0 {
            current.set(index - 1);
            playing.set(true);
        } else if len > 0 && repeat.get_untracked() == RepeatMode::All {
            current.set(len - 1);
            playing.set(true);
        }
    };
    let next = move |_| {
        let next = next_queue_index(
            tracks.get_untracked().len(),
            current.get_untracked(),
            shuffle.get_untracked(),
            repeat.get_untracked(),
            js_sys::Math::random(),
        );
        if let Some(next) = next {
            current.set(next);
            playing.set(true);
        } else {
            playing.set(false);
        }
    };
    view! {<footer class="player real-player">
        <Show when=move || queue_open.get()>
            <aside class="queue-panel" aria-label="Playback queue">
                <header><div><p class="eyebrow">"PLAYBACK"</p><h2>"Queue"</h2></div><div><span>{move || format!("{} TRACKS", tracks.get().len())}</span><button type="button" on:click=move |_| queue_open.set(false) aria-label="Close playback queue">"×"</button></div></header>
                <Show when=move || !tracks.get().is_empty() fallback=move || view! {<div class="queue-empty">"The playback queue is empty."</div>}>
                    <div class="queue-items">{move || tracks.get().into_iter().enumerate().map(|(idx, item)| {
                        let playback_track = item.clone();
                        view! {
                        <div class="queue-item" class:current=move || current.get() == idx>
                            <button type="button" class="queue-select" on:click=move |_| { current.set(idx); playing.set(true); start_track_playback(&playback_track); }><span>{format!("{:02}", idx + 1)}</span><span><b>{item.title}</b><small>{format!("{} · {}", item.artist.unwrap_or_else(|| "Unknown artist".into()), item.source_name)}</small></span></button>
                            <button type="button" class="queue-remove" aria-label="Remove track from queue" on:click=move |_| {
                                let old_current = current.get_untracked();
                                tracks.update(|items| { if idx < items.len() { items.remove(idx); } });
                                let len = tracks.get_untracked().len();
                                if len == 0 { current.set(0); playing.set(false); }
                                else if idx < old_current { current.set(old_current - 1); }
                                else if old_current >= len { current.set(len - 1); }
                            }>"REMOVE"</button>
                        </div>
                    }}).collect_view()}</div>
                    <button type="button" class="queue-clear" on:click=move |_| { tracks.set(Vec::new()); current.set(0); playing.set(false); }>"Clear queue"</button>
                </Show>
            </aside>
        </Show>
        <div class="now"><div><b>{move||track().map(|t|t.title).unwrap_or_else(||"Nothing playing".into())}</b><span>{move||track().map(|t|format!("{} · {}",t.artist.unwrap_or_default(),t.source_name)).unwrap_or_default()}</span></div></div>
        <div class="custom-player">
            <audio id="resonance-audio" class="player-audio" node_ref=audio_ref
                on:loadedmetadata=move |event| {
                    let audio = event_target::<web_sys::HtmlAudioElement>(&event);
                    duration.set(audio.duration());
                    audio.set_volume(volume.get_untracked());
                    audio.set_muted(muted.get_untracked());
                    let restored = normalized_position(
                        position.get_untracked(),
                        audio.duration().is_finite().then(|| audio.duration().floor().max(0.0) as u64),
                    );
                    if restored > 0.0 { audio.set_current_time(restored); }
                    position.set(restored);
                    persisted_second.set(restored.floor() as u64);
                    if playing.get_untracked() { let _ = audio.play(); }
                }
                on:timeupdate=move |event| {
                    let seconds = event_target::<web_sys::HtmlAudioElement>(&event).current_time();
                    position.set(seconds);
                    let previous = last_media_position.get_untracked();
                    last_media_position.set(Some(seconds));
                    if playing.get_untracked() {
                        listened_seconds.update(|total| *total += forward_playback_delta(previous, seconds));
                        if scrobbling.get_untracked() && should_submit_scrobble(listened_seconds.get_untracked(), duration.get_untracked(), submission_sent.get_untracked()) {
                            if let Some(track) = tracks.get_untracked().get(current.get_untracked()).cloned() {
                                let supported = providers.get_untracked().into_iter().find(|provider| provider.id == track.id.provider_id).is_some_and(|provider| provider.scrobbling_supported);
                                if supported {
                                    submission_sent.set(true);
                                    let started_ms = scrobble_started_ms.get_untracked();
                                    spawn_local(async move { if let Err(message) = report_scrobble(track.id, true, started_ms).await { scrobble_error.set(Some(format!("Scrobbling failed: {message}"))); } });
                                }
                            }
                        }
                    }
                    let whole_seconds = seconds.floor().max(0.0) as u64;
                    if whole_seconds / 5 != persisted_second.get_untracked() / 5 {
                        persisted_second.set(whole_seconds);
                        persist_playback(
                            &tracks.get_untracked(), current.get_untracked(), seconds,
                            volume.get_untracked(), muted.get_untracked(), shuffle.get_untracked(),
                            repeat.get_untracked(),
                        );
                    }
                }
                on:durationchange=move |event| duration.set(event_target::<web_sys::HtmlAudioElement>(&event).duration())
                on:playing=move |_| {
                    playing.set(true);
                    last_media_position.set(Some(position.get_untracked()));
                    if scrobbling.get_untracked() && !now_playing_sent.get_untracked() {
                        if let Some(track) = tracks.get_untracked().get(current.get_untracked()).cloned() {
                            let supported = providers.get_untracked().into_iter().find(|provider| provider.id == track.id.provider_id).is_some_and(|provider| provider.scrobbling_supported);
                            if supported {
                                now_playing_sent.set(true);
                                let started_ms = scrobble_started_ms.get_untracked();
                                spawn_local(async move { if let Err(message) = report_scrobble(track.id, false, started_ms).await { scrobble_error.set(Some(format!("Now-playing report failed: {message}"))); } });
                            }
                        }
                    }
                }
                on:pause=move |event| {
                    let audio = event_target::<web_sys::HtmlAudioElement>(&event);
                    if audio.ready_state() == 0 { return; }
                    let seconds = audio.current_time();
                    position.set(seconds);
                    persist_playback(
                        &tracks.get_untracked(), current.get_untracked(), seconds,
                        volume.get_untracked(), muted.get_untracked(), shuffle.get_untracked(),
                        repeat.get_untracked(),
                    );
                }
                on:ended=move |event| {
        if scrobbling.get_untracked() && !submission_sent.get_untracked() {
            if let Some(track) = tracks.get_untracked().get(current.get_untracked()).cloned() {
                let supported = providers.get_untracked().into_iter().find(|provider| provider.id == track.id.provider_id).is_some_and(|provider| provider.scrobbling_supported);
                if supported {
                    submission_sent.set(true);
                    let started_ms = scrobble_started_ms.get_untracked();
                    spawn_local(async move { if let Err(message) = report_scrobble(track.id, true, started_ms).await { scrobble_error.set(Some(format!("Scrobbling failed: {message}"))); } });
                }
            }
        }
        now_playing_sent.set(false);
        submission_sent.set(false);
        listened_seconds.set(0.0);
        last_media_position.set(None);
        scrobble_started_ms.set(js_sys::Date::now().max(0.0) as u64);
        if repeat.get_untracked() == RepeatMode::One {
            let audio = event_target::<web_sys::HtmlAudioElement>(&event);
            audio.set_current_time(0.0);
            position.set(0.0);
            let _ = audio.play();
            playing.set(true);
            return;
        }
        let len = tracks.get_untracked().len();
        if len == 0 {
            playing.set(false);
            return;
        }
        let next = next_queue_index(len, current.get_untracked(), shuffle.get_untracked(), repeat.get_untracked(), js_sys::Math::random());
        if let Some(next) = next {
            current.set(next);
            playing.set(true);
        } else {
            playing.set(false);
        }
    }></audio>
            <div class="transport-controls">
                <button type="button" title="Previous track" aria-label="Previous track" disabled=move || tracks.get().is_empty() on:click=previous>"◀◀"</button>
                <button type="button" class="play-toggle" title=move || if playing.get() {"Pause"} else {"Play"} aria-label=move || if playing.get() {"Pause"} else {"Play"} disabled=move || tracks.get().is_empty() on:click=move |_| {
                    if let Some(audio) = audio_ref.get_untracked() {
                        if playing.get_untracked() {
                            playing.set(false);
                            audio.pause().ok();
                        } else {
                            playing.set(true);
                            let _ = audio.play();
                        }
                    }
                }>{move || if playing.get() {"Ⅱ"} else {"▶"}}</button>
                <button type="button" title="Next track" aria-label="Next track" disabled=move || tracks.get().is_empty() on:click=next>"▶▶"</button>
            </div>
            <div class="player-timeline"><span>{move || player_time(position.get())}</span><input type="range" min="0" max=move || duration.get().max(0.0) step="0.1" prop:value=move || position.get() aria-label="Seek" disabled=move || tracks.get().is_empty() on:input=move |event| {
                if let Ok(seconds) = event_target_value(&event).parse::<f64>() {
                    if let Some(audio) = audio_ref.get_untracked() { audio.set_current_time(seconds); }
                    position.set(seconds);
                }
            }/><span>{move || player_time(duration.get())}</span></div>
            <div class="player-modes">
                <button type="button" class="icon-button" class:active=move || shuffle.get() aria-pressed=move || shuffle.get().to_string() aria-label=move || if shuffle.get() {"Disable shuffle"} else {"Enable shuffle"} title=move || if shuffle.get() {"Disable shuffle"} else {"Enable shuffle"} on:click=move |_| shuffle.update(|enabled| *enabled = !*enabled)>{shuffle_icon()}</button>
                <button type="button" class="icon-button" class:active=move || repeat.get() != RepeatMode::Off aria-label=move || match repeat.get() { RepeatMode::Off => "Repeat off; activate repeat one", RepeatMode::One => "Repeat one; activate repeat all", RepeatMode::All => "Repeat all; turn repeat off" } title=move || match repeat.get() { RepeatMode::Off => "Repeat off", RepeatMode::One => "Repeat one", RepeatMode::All => "Repeat all" } on:click=move |_| repeat.update(|mode| *mode = match *mode { RepeatMode::Off => RepeatMode::One, RepeatMode::One => RepeatMode::All, RepeatMode::All => RepeatMode::Off })>{move || repeat_icon(repeat.get())}</button>
                <button type="button" class="icon-button" class:active=move || muted.get() aria-pressed=move || muted.get().to_string() aria-label=move || if muted.get() {"Unmute"} else {"Mute"} title=move || if muted.get() {"Unmute"} else {"Mute"} on:click=move |_| {
                    muted.update(|value| *value = !*value);
                    if let Some(audio) = audio_ref.get_untracked() { audio.set_muted(muted.get_untracked()); }
                }>{move || volume_icon(muted.get())}</button>
                <input type="range" min="0" max="1" step="0.01" prop:value=move || volume.get() aria-label="Volume" on:input=move |event| {
                    if let Ok(level) = event_target_value(&event).parse::<f64>() {
                        volume.set(level);
                        muted.set(false);
                        if let Some(audio) = audio_ref.get_untracked() { audio.set_volume(level); audio.set_muted(false); }
                    }
                }/>
                <button type="button" class:active=move || queue_open.get() aria-expanded=move || queue_open.get().to_string() on:click=move |_| queue_open.update(|open| *open = !*open)>{move || format!("QUEUE {}", tracks.get().len())}</button>
            </div>
        </div>
    </footer>}
}

#[component]
pub fn App() -> impl IntoView {
    let QueueSnapshot {
        tracks: saved_tracks,
        current: saved_current,
        position_seconds: saved_position,
        volume: saved_volume,
        muted: saved_muted,
        shuffle: saved_shuffle,
        repeat: saved_repeat,
        ..
    } = load_queue_snapshot();
    let active = RwSignal::new(Page::Home);
    let providers = RwSignal::new(Vec::<Provider>::new());
    let albums = RwSignal::new(Vec::<Album>::new());
    let tracks = RwSignal::new(saved_tracks);
    let current = RwSignal::new(saved_current);
    let playing = RwSignal::new(false);
    let position = RwSignal::new(saved_position);
    let volume = RwSignal::new(saved_volume);
    let muted = RwSignal::new(saved_muted);
    let shuffle = RwSignal::new(saved_shuffle);
    let repeat = RwSignal::new(saved_repeat);
    let loading = RwSignal::new(false);
    let issues = RwSignal::new(Vec::<ProviderIssue>::new());
    let error = RwSignal::new(None::<String>);
    let favorites = RwSignal::new(FavoriteCollection::default());
    let scrobbling = RwSignal::new(load_scrobbling_enabled());
    provide_context(providers);
    provide_context(favorites);
    provide_context(error);
    provide_context(scrobbling);
    let active_track_id = RwSignal::new(
        tracks
            .get_untracked()
            .get(current.get_untracked())
            .map(|track| track.id.clone()),
    );
    Effect::new(move |_| {
        let items = tracks.get();
        let selected = current.get().min(items.len().saturating_sub(1));
        let next_track_id = items.get(selected).map(|track| track.id.clone());
        if active_track_id.get_untracked() != next_track_id {
            active_track_id.set(next_track_id);
            position.set(0.0);
        }
        persist_playback(
            &items,
            selected,
            position.get_untracked(),
            volume.get(),
            muted.get(),
            shuffle.get(),
            repeat.get(),
        );
    });
    if let Some(window) = web_sys::window() {
        let save_before_unload = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            persist_playback(
                &tracks.get_untracked(),
                current.get_untracked(),
                position.get_untracked(),
                volume.get_untracked(),
                muted.get_untracked(),
                shuffle.get_untracked(),
                repeat.get_untracked(),
            );
        });
        let _ = window.add_event_listener_with_callback(
            "beforeunload",
            save_before_unload.as_ref().unchecked_ref(),
        );
        save_before_unload.forget();
    }
    spawn_local(async move {
        match get_json::<Vec<Provider>>("/providers").await {
            Ok(items) => providers.set(items),
            Err(message) => error.set(Some(message)),
        }
    });
    Effect::new(move |_| {
        let provider_count = providers.get().len();
        if provider_count == 0 {
            albums.set(Vec::new());
            issues.set(Vec::new());
            favorites.set(FavoriteCollection::default());
            return;
        }
        spawn_local(load_library(albums, loading, issues, error));
        spawn_local(async move {
            match get_json::<FavoriteCollection>("/library/favorites").await {
                Ok(items) => favorites.set(items),
                Err(message) => error.set(Some(message)),
            }
        });
    });
    view! {<div class="app"><Nav active providers/><div class="view">{move||match active.get(){Page::Home=>view!{<Home providers albums queue=tracks current playing loading issues error/>}.into_any(),Page::Albums=>view!{<AlbumsPage providers albums tracks current playing loading issues error/>}.into_any(),Page::Artists=>view!{<ArtistsPage providers queue=tracks current playing/>}.into_any(),Page::Playlists=>view!{<PlaylistsPage providers queue=tracks current playing/>}.into_any(),Page::Favorites=>view!{<FavoritesPage providers queue=tracks current playing/>}.into_any(),Page::Search=>view!{<SearchPage providers queue=tracks current playing/>}.into_any(),Page::Settings=>view!{<Settings providers/>}.into_any()}}</div><nav class="mobile-nav">{[(Page::Home,"⌂","Home"),(Page::Albums,"▣","Albums"),(Page::Favorites,"★","Favorites"),(Page::Search,"⌕","Search"),(Page::Settings,"⚙","Settings")].into_iter().map(|(p,g,l)|view!{<button class:active=move||active.get()==p on:click=move |_|active.set(p)><span>{g}</span>{l}</button>}).collect_view()}</nav><Player tracks current playing position volume muted shuffle repeat/></div>}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[cfg(test)]
mod tests {
    use super::{
        forward_playback_delta, next_queue_index, normalized_position, normalized_volume,
        page_count, scrobble_threshold, should_submit_scrobble, QueueSnapshot, RepeatMode,
    };

    #[test]
    fn scrobble_threshold_is_half_the_track_or_four_minutes() {
        assert_eq!(scrobble_threshold(180.0), 90.0);
        assert_eq!(scrobble_threshold(600.0), 240.0);
        assert_eq!(scrobble_threshold(f64::NAN), 240.0);
    }

    #[test]
    fn scrobble_progress_ignores_seeks_and_duplicate_submissions() {
        assert_eq!(forward_playback_delta(Some(10.0), 11.5), 1.5);
        assert_eq!(forward_playback_delta(Some(10.0), 100.0), 0.0);
        assert_eq!(forward_playback_delta(Some(100.0), 10.0), 0.0);
        assert!(should_submit_scrobble(90.0, 180.0, false));
        assert!(!should_submit_scrobble(90.0, 180.0, true));
    }

    #[test]
    fn album_page_count_handles_empty_and_partial_pages() {
        assert_eq!(page_count(0, 24), 1);
        assert_eq!(page_count(24, 24), 1);
        assert_eq!(page_count(25, 24), 2);
        assert_eq!(page_count(81, 24), 4);
    }

    #[test]
    fn queue_navigation_stops_or_wraps_at_the_boundary() {
        assert_eq!(next_queue_index(3, 2, false, RepeatMode::Off, 0.5), None);
        assert_eq!(next_queue_index(3, 2, false, RepeatMode::All, 0.5), Some(0));
    }

    #[test]
    fn non_repeating_shuffle_never_wraps_backwards() {
        assert_eq!(next_queue_index(5, 2, true, RepeatMode::Off, 0.0), Some(3));
        assert_eq!(next_queue_index(5, 2, true, RepeatMode::Off, 0.99), Some(4));
        assert_eq!(next_queue_index(5, 4, true, RepeatMode::Off, 0.5), None);
    }

    #[test]
    fn legacy_queue_snapshots_receive_safe_playback_defaults() {
        let snapshot: QueueSnapshot =
            serde_json::from_str(r#"{"version":1,"tracks":[],"current":0}"#)
                .expect("v1 queue snapshot should remain readable");

        assert_eq!(snapshot.position_seconds, 0.0);
        assert_eq!(snapshot.volume, 1.0);
        assert!(!snapshot.muted);
        assert!(!snapshot.shuffle);
        assert_eq!(snapshot.repeat, RepeatMode::Off);
    }

    #[test]
    fn invalid_or_completed_positions_restart_safely() {
        assert_eq!(normalized_position(f64::NAN, Some(180)), 0.0);
        assert_eq!(normalized_position(-1.0, Some(180)), 0.0);
        assert_eq!(normalized_position(178.0, Some(180)), 0.0);
        assert_eq!(normalized_position(63.5, Some(180)), 63.5);
    }

    #[test]
    fn invalid_volume_values_are_normalized() {
        assert_eq!(normalized_volume(f64::NAN), 1.0);
        assert_eq!(normalized_volume(-0.5), 0.0);
        assert_eq!(normalized_volume(1.5), 1.0);
        assert_eq!(normalized_volume(0.45), 0.45);
    }

    #[test]
    fn malformed_snapshots_are_rejected() {
        assert!(serde_json::from_str::<QueueSnapshot>("not json").is_err());
    }
}
