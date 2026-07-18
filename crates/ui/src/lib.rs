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

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Album {
    id: String,
    name: String,
    artist: Option<String>,
    cover_art: Option<String>,
    song_count: Option<u32>,
    duration_seconds: Option<u64>,
    year: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Track {
    id: String,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    cover_art: Option<String>,
    duration_seconds: Option<u64>,
    track_number: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumDetail {
    #[serde(flatten)]
    #[allow(dead_code)]
    album: Album,
    tracks: Vec<Track>,
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
    provider_id: String,
    albums: RwSignal<Vec<Album>>,
    tracks: RwSignal<Vec<Track>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    error.set(None);
    match get_json::<Vec<Album>>(&format!(
        "/providers/{}/albums?limit=30",
        encode(&provider_id)
    ))
    .await
    {
        Ok(items) => {
            let first = items.first().map(|album| album.id.clone());
            albums.set(items);
            if let Some(album_id) = first {
                match get_json::<AlbumDetail>(&format!(
                    "/providers/{}/albums/{}",
                    encode(&provider_id),
                    encode(&album_id)
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
            error.set(Some(message));
        }
    }
    loading.set(false);
}

fn cover_url(provider: &str, cover: Option<&str>, size: u32) -> Option<String> {
    cover.map(|id| {
        api(&format!(
            "/providers/{}/covers/{}?size={size}",
            encode(provider),
            encode(id)
        ))
    })
}

#[component]
fn Artwork(
    provider: String,
    cover: Option<String>,
    title: String,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let src = cover_url(&provider, cover.as_deref(), 500);
    view! { <div class=format!("cover {class}")>{src.map(|src| view!{<img src=src alt=format!("Cover for {title}")/>})}<span>{title}</span></div> }
}

#[component]
fn Nav(
    active: RwSignal<Page>,
    providers: RwSignal<Vec<Provider>>,
    selected: RwSignal<String>,
) -> impl IntoView {
    let item = move |page, label, glyph| view! { <button class:active=move||active.get()==page on:click=move |_|active.set(page)><span class="nav-icon">{glyph}</span><span>{label}</span></button> };
    view! {
        <aside class="sidebar">
            <div class="brand"><span class="brand-mark">"R"</span><div><strong>"RESONANCE"</strong><small>"SUBSONIC CLIENT"</small></div></div>
            <nav aria-label="Primary navigation">{item(Page::Home,"Home","⌂")}{item(Page::Albums,"Albums","▣")}{item(Page::Artists,"Artists","◉")}{item(Page::Playlists,"Playlists","≡")}{item(Page::Search,"Search","⌕")}{item(Page::Settings,"Settings","⚙")}</nav>
            <section class="sources"><p class="eyebrow">"SOURCES"</p><div class="source-list">
                {move||providers.get().into_iter().map(|p|{let id=p.id.clone();let active_id=id.clone();view!{<button class:active=move||selected.get()==active_id on:click=move |_|{selected.set(id.clone());active.set(Page::Home)}><i class="online"></i><span><b>{p.name}</b><small>{format!("{} · {}",p.server_type,p.server_version)}</small></span><em>"▥"</em></button>}}).collect_view()}
                <Show when=move||providers.get().is_empty()><p class="empty-source">"No connected servers"</p></Show>
            </div></section>
        </aside>
        <header class="mobile-head"><div class="brand"><span class="brand-mark">"R"</span><strong>"RESONANCE"</strong></div><button aria-label="Settings" on:click=move |_|active.set(Page::Settings)>"⚙"</button></header>
    }
}

#[component]
fn Home(
    selected: RwSignal<String>,
    albums: RwSignal<Vec<Album>>,
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let run_search = move |_| {
        let provider = selected.get();
        let term = query.get();
        if provider.is_empty() || term.trim().is_empty() {
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match get_json::<Vec<Track>>(&format!(
                "/providers/{}/search?q={}&limit=50",
                encode(&provider),
                encode(&term)
            ))
            .await
            {
                Ok(items) => tracks.set(items),
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    };
    view! { <main class="content">
        <header class="editorial-head"><div><p class="eyebrow">"YOUR ACTIVE SOURCE"</p><h1>"Your library"</h1><p>"Albums and tracks served directly by your selected provider."</p></div><form class="search" on:submit=move|e|{e.prevent_default();run_search(())}><span>"⌕"</span><input aria-label="Search library" placeholder="Search this server" on:input=move|e|query.set(event_target_value(&e))/></form></header>
        <Show when=move||!selected.get().is_empty() fallback=||view!{<section class="empty-library"><h2>"Connect a music server"</h2><p>"Open Settings → Connections and add your Subsonic server to begin."</p></section>}>
            <Show when=move||loading.get()><p class="library-state">"Tuning into your server…"</p></Show>
            <Show when=move||error.get().is_some()>{move||error.get().map(|message|view!{<div class="library-error"><b>"Could not load this library"</b><p>{message}</p></div>})}</Show>
            <Show when=move||!albums.get().is_empty()>{move||albums.get().first().cloned().map(|album|{let provider=selected.get();view!{<section class="feature real-feature"><Artwork provider=provider cover=album.cover_art.clone() title=album.name.clone() class="feature-art"/><div class="feature-copy"><p class="eyebrow coral">"RECENTLY ADDED"</p><h2>{album.name}</h2><p class="feature-artist">{format!("{} · {}",album.artist.unwrap_or_else(||"Unknown artist".into()),album.year.map(|y|y.to_string()).unwrap_or_default())}</p><p class="description">{format!("{} tracks from your active Subsonic library.",album.song_count.unwrap_or(0))}</p></div></section>}})}</Show>
            <section class="recent"><div class="section-title"><div><p class="eyebrow">"FROM THE SERVER"</p><h2>"Recent albums"</h2></div><span class="track-count">{move||format!("{} ALBUMS",albums.get().len())}</span></div><div class="album-row real-albums">{move||albums.get().into_iter().map(|album|{let provider=selected.get();let album_id=album.id.clone();view!{<button class="album" on:click=move |_|{let provider=provider.clone();let album_id=album_id.clone();loading.set(true);spawn_local(async move{match get_json::<AlbumDetail>(&format!("/providers/{}/albums/{}",encode(&provider),encode(&album_id))).await{Ok(detail)=>tracks.set(detail.tracks),Err(message)=>error.set(Some(message))}loading.set(false);});}><Artwork provider=provider.clone() cover=album.cover_art.clone() title=album.name.clone()/><strong>{album.name}</strong><span>{album.artist.unwrap_or_else(||"Unknown artist".into())}</span></button>}}).collect_view()}</div></section>
            <section class="tracks"><div class="section-title"><div><p class="eyebrow">"SELECTED ALBUM / SEARCH"</p><h2>"Tracks"</h2></div><span class="track-count">{move||format!("{} TRACKS",tracks.get().len())}</span></div><div class="track-list">{move||tracks.get().into_iter().enumerate().map(|(idx,track)|view!{<button class:current=move||current.get()==idx on:click=move |_|{current.set(idx);playing.set(true)}><span class="track-no">{track.track_number.map(|n|format!("{n:02}")).unwrap_or_else(||"—".into())}</span><span class="mini-cover"></span><span class="track-name"><b>{track.title}</b><small>{track.artist.unwrap_or_else(||"Unknown artist".into())}</small></span><span class="track-album">{track.album.unwrap_or_default()}</span><span class="track-time">{format_duration(track.duration_seconds)}</span></button>}).collect_view()}</div></section>
        </Show>
    </main> }
}

fn format_duration(seconds: Option<u64>) -> String {
    seconds
        .map(|s| format!("{}:{:02}", s / 60, s % 60))
        .unwrap_or_else(|| "—".into())
}

#[component]
fn Settings(
    providers: RwSignal<Vec<Provider>>,
    selected: RwSignal<String>,
    active: RwSignal<Page>,
) -> impl IntoView {
    let connections = RwSignal::new(true);
    let dialog = RwSignal::new(false);
    let action_error = RwSignal::new(None::<String>);
    view! {<main class="content settings"><header><p class="eyebrow coral">"SYSTEM & SOURCES"</p><h1>"Settings"</h1><p>"Manage the servers Resonance can use."</p></header><div class="settings-tabs" role="tablist"><button role="tab" aria-selected=move||(!connections.get()).to_string() class:active=move||!connections.get() on:click=move |_|connections.set(false)>"General"</button><button role="tab" aria-selected=move||connections.get().to_string() class:active=move||connections.get() on:click=move |_|connections.set(true)>"Connections"</button></div>
    <Show when=move||connections.get() fallback=||view!{<section class="general"><div class="preference"><div><b>"Streaming quality"</b><small>"Server transcoding preferences are coming next."</small></div><select><option>"Original"</option></select></div></section>}>
        <section class="connections"><div class="connections-head"><div><p class="eyebrow">"LIVE PROVIDERS"</p><h2>"Connections"</h2><p>"Credentials are verified by the Rust backend before a server is added."</p></div><button class="primary" on:click=move |_|dialog.set(true)>"＋ Add Provider"</button></div>
        <Show when=move||action_error.get().is_some()>{move||action_error.get().map(|e|view!{<div class="library-error"><p>{e}</p></div>})}</Show>
        <div class="provider-list">{move||providers.get().into_iter().map(|p|{let select_id=p.id.clone();let remove_id=p.id.clone();view!{<article class:active=move||selected.get()==select_id><span class="signal"><i></i><i></i><i></i></span><div class="provider-identity"><b>{p.name}</b><small>{p.url}</small></div><dl><div><dt>"AUTH"</dt><dd>{p.auth}</dd></div><div><dt>"SERVER"</dt><dd class="connected">{p.server_type}</dd></div></dl><button on:click={let id=p.id.clone();move |_|{selected.set(id.clone());active.set(Page::Home)}}>"Use"</button><button class="remove" on:click=move |_|{let id=remove_id.clone();spawn_local(async move{match Request::delete(&api(&format!("/providers/{}",encode(&id)))).send().await{Ok(response) if response.ok()=>{providers.update(|items|items.retain(|x|x.id!=id));if selected.get()==id{selected.set(providers.get().first().map(|p|p.id.clone()).unwrap_or_default())}},Ok(response)=>action_error.set(Some(response.text().await.unwrap_or_else(|_|"Could not remove provider".into()))),Err(e)=>action_error.set(Some(e.to_string()))}});}>"Remove"</button></article>}}).collect_view()}</div>
        <Show when=move||providers.get().is_empty()><div class="empty-library"><h2>"No providers connected"</h2><p>"Add your first Subsonic server above."</p></div></Show>
        <p class="prototype-note">"Web credentials are held only in backend memory and disappear when the backend stops. Native credential-vault persistence is not enabled yet."</p></section>
    </Show><ProviderDialog open=dialog providers selected error=action_error/></main>}
}

#[component]
fn ProviderDialog(
    open: RwSignal<bool>,
    providers: RwSignal<Vec<Provider>>,
    selected: RwSignal<String>,
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
    view! {<Show when=move||open.get()><dialog node_ref=dialog_ref class="dialog provider-dialog" on:cancel=move |_: leptos::ev::Event|open.set(false)><button class="dialog-close" on:click=move |_|open.set(false)>"×"</button><p class="eyebrow coral">"NEW CONNECTION"</p><h2>"Add Provider"</h2><p>"Resonance will verify these details before saving the connection."</p><form on:submit=move|ev|{ev.prevent_default();saving.set(true);error.set(None);let request=RegisterProvider{name:name.get(),url:url.get(),username:Some(username.get()),auth:if api_key.get(){AuthMethod::ApiKey}else{AuthMethod::Password},secret:secret.get()};spawn_local(async move{let builder=match Request::post(&api("/providers")).json(&request){Ok(v)=>v,Err(e)=>{error.set(Some(e.to_string()));saving.set(false);return}};match builder.send().await{Ok(response) if response.ok()=>match response.json::<Provider>().await{Ok(provider)=>{selected.set(provider.id.clone());providers.update(|items|items.push(provider));secret.set(String::new());open.set(false)},Err(e)=>error.set(Some(e.to_string()))},Ok(response)=>error.set(Some(response.text().await.unwrap_or_else(|_|"Connection failed".into()))),Err(e)=>error.set(Some(format!("Cannot reach backend: {e}")))}saving.set(false);});}><label>"Name"<input autofocus required placeholder="Home library" on:input=move|e|name.set(event_target_value(&e))/></label><label>"Server URL"<input required type="url" placeholder="https://music.example.com" on:input=move|e|url.set(event_target_value(&e))/></label><label>"Username"<input required=!api_key.get() autocomplete="username" on:input=move|e|username.set(event_target_value(&e))/></label><div class="auth-toggle"><button type="button" class:active=move||!api_key.get() on:click=move |_|api_key.set(false)>"Password"</button><button type="button" class:active=move||api_key.get() on:click=move |_|api_key.set(true)>"API Key"</button></div><label>{move||if api_key.get(){"API Key"}else{"Password"}}<input required type="password" autocomplete="new-password" on:input=move|e|secret.set(event_target_value(&e))/></label><p class="security">"Credentials are sent to the local Rust backend and are not stored in browser storage."</p><div class="dialog-actions"><button type="button" on:click=move |_|open.set(false)>"Cancel"</button><button class="primary" type="submit" disabled=move||saving.get()>{move||if saving.get(){"Connecting…"}else{"Save Provider"}}</button></div></form></dialog></Show>}
}

#[component]
fn Player(
    selected: RwSignal<String>,
    tracks: RwSignal<Vec<Track>>,
    current: RwSignal<usize>,
    playing: RwSignal<bool>,
) -> impl IntoView {
    let track = move || tracks.get().get(current.get()).cloned();
    view! {<footer class="player real-player"><div class="now"><div><b>{move||track().map(|t|t.title).unwrap_or_else(||"Nothing playing".into())}</b><span>{move||track().and_then(|t|t.artist).unwrap_or_default()}</span></div></div><div class="transport">{move||track().map(|t|{let src=api(&format!("/providers/{}/tracks/{}/stream",encode(&selected.get()),encode(&t.id)));view!{<audio controls autoplay=playing.get() src=src></audio>}})}</div><div class="player-tools"><span>{move||if playing.get(){"STREAMING"}else{"READY"}}</span></div></footer>}
}

#[component]
pub fn App() -> impl IntoView {
    let active = RwSignal::new(Page::Home);
    let providers = RwSignal::new(Vec::<Provider>::new());
    let selected = RwSignal::new(String::new());
    let albums = RwSignal::new(Vec::<Album>::new());
    let tracks = RwSignal::new(Vec::<Track>::new());
    let current = RwSignal::new(0usize);
    let playing = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    spawn_local(async move {
        match get_json::<Vec<Provider>>("/providers").await {
            Ok(items) => {
                if let Some(first) = items.first() {
                    selected.set(first.id.clone())
                }
                providers.set(items)
            }
            Err(message) => error.set(Some(message)),
        }
    });
    Effect::new(move |_| {
        let id = selected.get();
        current.set(0);
        playing.set(false);
        if !id.is_empty() {
            spawn_local(load_library(id, albums, tracks, loading, error));
        } else {
            albums.set(Vec::new());
            tracks.set(Vec::new());
        }
    });
    view! {<div class="app"><Nav active providers selected/><div class="view">{move||match active.get(){Page::Home=>view!{<Home selected albums tracks current playing loading error/>}.into_any(),Page::Settings=>view!{<Settings providers selected active/>}.into_any(),_=>view!{<main class="content placeholder"><p class="eyebrow">"LIBRARY"</p><h1>"Coming next"</h1><p>"Use Home to browse the connected server while this view is implemented."</p></main>}.into_any()}}</div><nav class="mobile-nav">{[(Page::Home,"⌂","Home"),(Page::Albums,"▣","Albums"),(Page::Search,"⌕","Search"),(Page::Settings,"⚙","Settings")].into_iter().map(|(p,g,l)|view!{<button class:active=move||active.get()==p on:click=move |_|active.set(p)><span>{g}</span>{l}</button>}).collect_view()}</nav><Player selected tracks current playing/></div>}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
