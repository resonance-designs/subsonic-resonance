use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

const API_BASE: &str = "http://127.0.0.1:3000/api";

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Albums,
    Artists,
    Playlists,
    Search,
    Settings,
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
    tracks: RwSignal<Vec<Track>>,
    loading: RwSignal<bool>,
    issues: RwSignal<Vec<ProviderIssue>>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    error.set(None);
    match get_json::<AggregateResponse<Album>>("/library/albums?limit=500").await {
        Ok(response) => {
            let first = response.items.first().map(|album| album.id.clone());
            albums.set(response.items);
            issues.set(response.issues);
            if let Some(id) = first {
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
            } else {
                tracks.set(Vec::new());
            }
        }
        Err(message) => {
            albums.set(Vec::new());
            tracks.set(Vec::new());
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
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    loading: RwSignal<bool>,
    issues: RwSignal<Vec<ProviderIssue>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
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
                    tracks.set(response.items);
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
            <Show when=move||!albums.get().is_empty()>{move||albums.get().first().cloned().map(|album|view!{<section class="feature real-feature"><Artwork cover=album.cover_art.clone() title=album.name.clone() class="feature-art"/><div class="feature-copy"><p class="eyebrow coral">"FROM THE UNIFIED LIBRARY"</p><h2>{album.name}</h2><p class="feature-artist">{format!("{} · {}",album.artist.unwrap_or_else(||"Unknown artist".into()),album.source_name)}</p><p class="description">{format!("{} tracks",album.song_count.unwrap_or(0))}</p></div></section>})}</Show>
            <section class="recent"><div class="section-title"><div><p class="eyebrow">"ALL SOURCES"</p><h2>"Albums"</h2></div><span class="track-count">{move||format!("{} ALBUMS",albums.get().len())}</span></div><div class="album-row real-albums">
                {move||albums.get().into_iter().map(|album|{let id=album.id.clone();view!{<button class="album" on:click=move |_|{let id=id.clone();loading.set(true);spawn_local(async move{match get_json::<AlbumDetail>(&format!("/library/albums/{}/{}",encode(&id.provider_id),encode(&id.item_id))).await{Ok(detail)=>tracks.set(detail.tracks),Err(message)=>error.set(Some(message))}loading.set(false);});}><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{format!("{} · {}",album.artist.unwrap_or_else(||"Unknown artist".into()),album.source_name)}</span></button>}}).collect_view()}
            </div></section>
            <section class="tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED ALBUM / SEARCH"</p><h2>"Tracks"</h2></div><span class="track-count">{move||format!("{} TRACKS",tracks.get().len())}</span></div><div class="track-list">
                {move||tracks.get().into_iter().enumerate().map(|(idx,track)|view!{<button class:current=move||current.get()==idx on:click=move |_|{current.set(idx);playing.set(true)}><span class="track-no">{track.track_number.map(|n|format!("{n:02}")).unwrap_or_else(||"—".into())}</span><span class="mini-cover"></span><span class="track-name"><b>{track.title}</b><small>{format!("{} · {}",track.artist.unwrap_or_else(||"Unknown artist".into()),track.source_name)}</small></span><span class="track-album">{track.album.unwrap_or_default()}</span><span class="track-time">{format_duration(track.duration_seconds)}</span></button>}).collect_view()}
            </div></section>
        </Show>
    </main>}
}

fn format_duration(seconds: Option<u64>) -> String {
    seconds
        .map(|s| format!("{}:{:02}", s / 60, s % 60))
        .unwrap_or_else(|| "—".into())
}

fn load_album_tracks(
    id: MediaId,
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    error.set(None);
    spawn_local(async move {
        match get_json::<AlbumDetail>(&format!(
            "/library/albums/{}/{}",
            encode(&id.provider_id),
            encode(&id.item_id)
        ))
        .await
        {
            Ok(detail) => {
                tracks.set(detail.tracks);
                current.set(0);
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
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
fn TrackResults(
    items: RwSignal<Vec<Track>>,
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
                    view! {
                        <button on:click=move |_| {
                            queue.set(result_items.get());
                            current.set(idx);
                            playing.set(true);
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
    let selected = RwSignal::new(None::<MediaId>);
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
                    view! {<button class="album-card" class:selected=move || selected.get().as_ref() == Some(&selected_id) on:click=move |_| {selected.set(Some(id.clone())); load_album_tracks(id.clone(), tracks, current, loading, error)}><Artwork cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(|| "Unknown artist".into())}</span><small>{format!("{}{}", album.source_name, album.year.map(|year| format!(" · {year}")).unwrap_or_default())}</small></button>}
                }).collect_view()}</div>
            </Show>
            <section class="tracks page-tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED ALBUM"</p><h2>"Tracks"</h2></div><span class="track-count">{move || if selected.get().is_some() { format!("{} TRACKS", tracks.get().len()) } else { "0 TRACKS".into() }}</span></div><Show when=move || selected.get().is_some() fallback=move || view! {<div class="empty-results"><p>"Select an album to view its tracks."</p></div>}><TrackResults items=tracks queue=tracks current playing empty_message="This album did not return any tracks."/></Show></section>
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
    let submit = move |_| {
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
        <header><p class="eyebrow">"ALL CONNECTED SOURCES"</p><h1>"Search"</h1><p>"Find tracks across the complete unified library."</p></header>
        <Show when=move || !providers.get().is_empty() fallback=|| view! {<section class="empty-library"><h2>"Connect a music server"</h2><p>"Add a provider in Settings before searching."</p></section>}>
            <form class="library-search" on:submit=move |event| {event.prevent_default(); submit(())}><span>"⌕"</span><input autofocus type="search" aria-label="Search all connected sources" placeholder="Track, artist, or album" on:input=move |event| query.set(event_target_value(&event))/><button class="primary" type="submit" disabled=move || loading.get()>{move || if loading.get() {"Searching…"} else {"Search"}}</button></form>
            <Show when=move || error.get().is_some()>{move || error.get().map(|message| view! {<div class="library-error"><b>"Search failed"</b><p>{message}</p></div>})}</Show>
            <ProviderNotices issues/>
            <Show when=move || searched.get()>
                <section class="tracks search-results"><div class="section-title"><div><p class="eyebrow">"SEARCH RESULTS"</p><h2>{move || format!("Results for “{}”", submitted_query.get())}</h2></div><span class="track-count">{move || format!("{} TRACKS", results.get().len())}</span></div><TrackResults items=results queue current playing empty_message="No tracks matched this search."/></section>
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
    view! {<main class="content settings"><header><p class="eyebrow coral">"SYSTEM & SOURCES"</p><h1>"Settings"</h1><p>"Every connection is available throughout Resonance."</p></header><div class="settings-tabs" role="tablist"><button role="tab" aria-selected=move||(!connections.get()).to_string() class:active=move||!connections.get() on:click=move |_|connections.set(false)>"General"</button><button role="tab" aria-selected=move||connections.get().to_string() class:active=move||connections.get() on:click=move |_|connections.set(true)>"Connections"</button></div>
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
    view! {<Show when=move||open.get()><dialog node_ref=dialog_ref class="dialog provider-dialog" on:cancel=move|_:leptos::ev::Event|open.set(false)><button class="dialog-close" on:click=move |_|open.set(false)>"×"</button><p class="eyebrow coral">"NEW CONNECTION"</p><h2>"Add Provider"</h2><p>"Resonance will verify these details before saving the connection."</p><form on:submit=move|ev|{ev.prevent_default();saving.set(true);error.set(None);let request=RegisterProvider{name:name.get(),url:url.get(),username:Some(username.get()),auth:if api_key.get(){AuthMethod::ApiKey}else{AuthMethod::Password},secret:secret.get()};spawn_local(async move{let builder=match Request::post(&api("/providers")).json(&request){Ok(v)=>v,Err(e)=>{error.set(Some(e.to_string()));saving.set(false);return}};match builder.send().await{Ok(response) if response.ok()=>match response.json::<Provider>().await{Ok(provider)=>{providers.update(|items|items.push(provider));secret.set(String::new());open.set(false)},Err(e)=>error.set(Some(e.to_string()))},Ok(response)=>error.set(Some(response.text().await.unwrap_or_else(|_|"Connection failed".into()))),Err(e)=>error.set(Some(format!("Cannot reach backend: {e}")))}saving.set(false);});}><label>"Name"<input autofocus required placeholder="Home library" on:input=move|e|name.set(event_target_value(&e))/></label><label>"Server URL"<input required type="url" placeholder="https://music.example.com" on:input=move|e|url.set(event_target_value(&e))/></label><label>"Username"<input required=!api_key.get() autocomplete="username" on:input=move|e|username.set(event_target_value(&e))/></label><div class="auth-toggle"><button type="button" class:active=move||!api_key.get() on:click=move |_|api_key.set(false)>"Password"</button><button type="button" class:active=move||api_key.get() on:click=move |_|api_key.set(true)>"API Key"</button></div><label>{move||if api_key.get(){"API Key"}else{"Password"}}<input required type="password" autocomplete="new-password" on:input=move|e|secret.set(event_target_value(&e))/></label><p class="security">"Credentials are sent to the local Rust backend and are not stored in browser storage."</p><div class="dialog-actions"><button type="button" on:click=move |_|open.set(false)>"Cancel"</button><button class="primary" type="submit" disabled=move||saving.get()>{move||if saving.get(){"Connecting…"}else{"Save Provider"}}</button></div></form></dialog></Show>}
}

#[component]
fn Player(
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let track = move || tracks.get().get(current.get()).cloned();
    view! {<footer class="player real-player"><div class="now"><div><b>{move||track().map(|t|t.title).unwrap_or_else(||"Nothing playing".into())}</b><span>{move||track().map(|t|format!("{} · {}",t.artist.unwrap_or_default(),t.source_name)).unwrap_or_default()}</span></div></div><div class="transport">{move||track().map(|t|{let src=api(&format!("/providers/{}/tracks/{}/stream",encode(&t.id.provider_id),encode(&t.id.item_id)));view!{<audio controls autoplay=playing.get() src=src></audio>}})}</div><div class="player-tools"><span>{move||if playing.get(){"STREAMING"}else{"READY"}}</span></div></footer>}
}

#[component]
pub fn App() -> impl IntoView {
    let active = RwSignal::new(Page::Home);
    let providers = RwSignal::new(Vec::<Provider>::new());
    let albums = RwSignal::new(Vec::<Album>::new());
    let tracks = RwSignal::new(Vec::<Track>::new());
    let current = RwSignal::new(0usize);
    let playing = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let issues = RwSignal::new(Vec::<ProviderIssue>::new());
    let error = RwSignal::new(None::<String>);
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
            tracks.set(Vec::new());
            issues.set(Vec::new());
            return;
        }
        spawn_local(load_library(albums, tracks, loading, issues, error));
    });
    view! {<div class="app"><Nav active providers/><div class="view">{move||match active.get(){Page::Home=>view!{<Home providers albums tracks current playing loading issues error/>}.into_any(),Page::Albums=>view!{<AlbumsPage providers albums tracks current playing loading issues error/>}.into_any(),Page::Search=>view!{<SearchPage providers queue=tracks current playing/>}.into_any(),Page::Settings=>view!{<Settings providers/>}.into_any(),_=>view!{<main class="content placeholder"><p class="eyebrow">"UNIFIED LIBRARY"</p><h1>"Coming next"</h1><p>"This page will use the new all-provider library service."</p></main>}.into_any()}}</div><nav class="mobile-nav">{[(Page::Home,"⌂","Home"),(Page::Albums,"▣","Albums"),(Page::Search,"⌕","Search"),(Page::Settings,"⚙","Settings")].into_iter().map(|(p,g,l)|view!{<button class:active=move||active.get()==p on:click=move |_|active.set(p)><span>{g}</span>{l}</button>}).collect_view()}</nav><Player tracks current playing/></div>}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
