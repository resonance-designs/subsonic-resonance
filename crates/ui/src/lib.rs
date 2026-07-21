use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

const API_BASE: &str = "http://127.0.0.1:3000/api";
const QUEUE_STORAGE_KEY: &str = "resonance.playbackQueue.v1";

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Albums,
    Artists,
    Playlists,
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
    open_subsonic: bool,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumDetail {
    #[allow(dead_code)]
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

fn api(path: &str) -> String {
    format!("{API_BASE}{path}")
}
fn encode(value: &str) -> String {
    js_sys::encode_uri_component(value)
        .as_string()
        .unwrap_or_default()
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
    match get_json::<AggregateResponse<Album>>("/library/albums?limit=500").await {
        Ok(response) => {
            albums.set(response.items);
            issues.set(response.issues);
        }
        Err(message) => {
            albums.set(Vec::new());
            issues.set(Vec::new());
            error.set(Some(message));
        }
    }
    loading.set(false);
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
            <nav aria-label="Primary navigation">{item(Page::Home,"Home","⌂")}{item(Page::Albums,"Albums","▣")}{item(Page::Artists,"Artists","◉")}{item(Page::Playlists,"Playlists","≡")}{item(Page::Search,"Search","⌕")}{item(Page::Settings,"Settings","⚙")}</nav>
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
                    view! {
                        <button on:click=move |_| {
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
                        </button>
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
    let selected = RwSignal::new(None::<MediaId>);
    let album_tracks = RwSignal::new(Vec::<Track>::new());
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

    view! {<main class="content library-page">
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
            <div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Releases"</h2></div><span class="track-count">{move || format!("{} SHOWN", visible_albums().len())}</span></div>
            <Show when=move || !visible_albums().is_empty() fallback=move || view! {<div class="empty-results"><p>"No albums match this filter."</p></div>}>
                <div class="album-grid">{move || visible_albums().into_iter().map(|album| {
                    let id = album.id.clone();
                    let selected_id = id.clone();
                    view! {<button class="media-card album-card" class:selected=move || selected.get().as_ref() == Some(&selected_id) disabled=move || loading.get() on:click=move |_| {
                        if loading.get_untracked() {
                            return;
                        }
                        loading.set(true);
                        selected.set(Some(id.clone()));
                        spawn_local(load_album_tracks(id.clone(), album_tracks, loading, error));
                    }><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(|| "Unknown artist".into())}</span><small>{format!("{}{}", album.source_name, album.year.map(|year| format!(" · {year}")).unwrap_or_default())}</small></button>}
                }).collect_view()}</div>
            </Show>
            <section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED ALBUM"</p><h2>"Tracks"</h2></div><span class="track-count">{move || if selected.get().is_some() { format!("{} TRACKS", album_tracks.get().len()) } else { "0 TRACKS".into() }}</span></div><Show when=move || selected.get().is_some() fallback=move || view! {<div class="empty-results"><p>"Select an album to view its tracks."</p></div>}><TrackResults items=album_tracks queue=tracks current playing empty_message="This album did not return any tracks."/></Show></section>
        </Show>
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
            <div><p class="eyebrow">"ARTIST · UNIFIED LIBRARY"</p><h1>{artist_name}</h1><p>{artist_source}</p></div>
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
fn Settings(providers: RwSignal<Vec<Provider>>) -> impl IntoView {
    let connections = RwSignal::new(true);
    let dialog = RwSignal::new(false);
    let action_error = RwSignal::new(None::<String>);
    view! {<main class="content settings"><header class="editorial-head"><div><p class="eyebrow">"SYSTEM & SOURCES"</p><h1>"Settings"</h1><p>"Every connection is available throughout Resonance."</p></div></header><div class="settings-tabs" role="tablist"><button role="tab" aria-selected=move||(!connections.get()).to_string() class:active=move||!connections.get() on:click=move |_|connections.set(false)>"General"</button><button role="tab" aria-selected=move||connections.get().to_string() class:active=move||connections.get() on:click=move |_|connections.set(true)>"Connections"</button></div>
        <Show when=move||connections.get() fallback=||view!{<section class="general"><div class="preference"><div><b>"Streaming quality"</b><small>"Server transcoding preferences are coming next."</small></div><select><option>"Original"</option></select></div></section>}>
            <section class="connections"><div class="connections-head"><div><p class="eyebrow">"LIVE PROVIDERS"</p><h2>"Connections"</h2><p>"All connected providers participate in the unified library."</p></div><button class="primary" on:click=move |_|dialog.set(true)>"＋ Add Provider"</button></div>
            <Show when=move||action_error.get().is_some()>{move||action_error.get().map(|e|view!{<div class="library-error"><p>{e}</p></div>})}</Show>
            <div class="provider-list">{move||providers.get().into_iter().map(|p|{let remove_id=p.id.clone();view!{<article class="active"><span class="signal"><i></i><i></i><i></i></span><div class="provider-identity"><b>{p.name}</b><small>{p.url}</small></div><dl><div><dt>"AUTH"</dt><dd>{p.auth}</dd></div><div><dt>"SERVER"</dt><dd class="connected">{p.server_type}</dd></div></dl><span class="connected">"Available everywhere"</span><button class="remove" on:click=move |_|{let id=remove_id.clone();spawn_local(async move{match Request::delete(&api(&format!("/providers/{}",encode(&id)))).send().await{Ok(response) if response.ok()=>providers.update(|items|items.retain(|x|x.id!=id)),Ok(response)=>action_error.set(Some(response.text().await.unwrap_or_else(|_|"Could not remove provider".into()))),Err(e)=>action_error.set(Some(e.to_string()))}});}>"Remove"</button></article>}}).collect_view()}</div>
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
    let track = move || tracks.get().get(current.get()).cloned();
    let queue_open = RwSignal::new(false);
    let duration = RwSignal::new(0.0_f64);
    let persisted_second = RwSignal::new(position.get_untracked().floor().max(0.0) as u64);
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
                on:play=move |_| playing.set(true)
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
            return;
        }
        spawn_local(load_library(albums, loading, issues, error));
    });
    view! {<div class="app"><Nav active providers/><div class="view">{move||match active.get(){Page::Home=>view!{<Home providers albums queue=tracks current playing loading issues error/>}.into_any(),Page::Albums=>view!{<AlbumsPage providers albums tracks current playing loading issues error/>}.into_any(),Page::Artists=>view!{<ArtistsPage providers queue=tracks current playing/>}.into_any(),Page::Playlists=>view!{<PlaylistsPage providers queue=tracks current playing/>}.into_any(),Page::Search=>view!{<SearchPage providers queue=tracks current playing/>}.into_any(),Page::Settings=>view!{<Settings providers/>}.into_any()}}</div><nav class="mobile-nav">{[(Page::Home,"⌂","Home"),(Page::Albums,"▣","Albums"),(Page::Search,"⌕","Search"),(Page::Settings,"⚙","Settings")].into_iter().map(|(p,g,l)|view!{<button class:active=move||active.get()==p on:click=move |_|active.set(p)><span>{g}</span>{l}</button>}).collect_view()}</nav><Player tracks current playing position volume muted shuffle repeat/></div>}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[cfg(test)]
mod tests {
    use super::{
        next_queue_index, normalized_position, normalized_volume, QueueSnapshot, RepeatMode,
    };

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
