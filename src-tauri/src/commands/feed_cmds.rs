use super::{
    EventTimelineResponse, FeedItemResponse, FeedRefreshRequest, FeedStateResponse, TagResponse,
};
use neotrix::neotrix::nt_expert_routing::moment_feed::{MomentContentType, MomentFeed};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{command, Emitter, State};

pub struct FeedEngine(pub Arc<Mutex<MomentFeed>>);

pub struct FeedStreamState(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

impl FeedStreamState {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl Default for FeedStreamState {
    fn default() -> Self {
        Self::new()
    }
}

fn content_type_to_string(ct: &MomentContentType) -> String {
    match ct {
        MomentContentType::Article => "article".to_string(),
        MomentContentType::Image => "image".to_string(),
        MomentContentType::Video => "video".to_string(),
        MomentContentType::Live => "live".to_string(),
        MomentContentType::Social => "social".to_string(),
    }
}

fn convert_item(
    item: &neotrix::neotrix::nt_expert_routing::moment_feed::FeedItem,
) -> FeedItemResponse {
    FeedItemResponse {
        id: item.id.clone(),
        title: item.title.clone(),
        description: item.description.clone(),
        content_type: content_type_to_string(&item.content_type),
        source_url: item.source_url.clone(),
        source_name: item.source_name.clone(),
        image_url: item.image_url.clone(),
        video_url: item.video_url.clone(),
        author: item.author.clone(),
        published_at: item.published_at as u64,
        score: item.score,
        tags: item.tags.clone(),
        neotrix_insight: item.neotrix_insight.clone(),
    }
}

fn convert_to_response(
    state: &neotrix::neotrix::nt_expert_routing::moment_feed::FeedState,
) -> FeedStateResponse {
    FeedStateResponse {
        items: state.items.iter().map(convert_item).collect(),
        timelines: state
            .timelines
            .iter()
            .map(|tl| EventTimelineResponse {
                id: tl.id.clone(),
                title: tl.title.clone(),
                item_ids: tl.items.iter().map(|i| i.id.clone()).collect(),
                start_time: tl.start_time as u64,
                end_time: tl.end_time as u64,
                key_events: tl.key_events.clone(),
                neotrix_summary: tl.neotrix_summary.clone(),
            })
            .collect(),
        tags: state
            .tags
            .iter()
            .map(|t| TagResponse {
                name: t.name.clone(),
                count: t.count as u64,
                is_active: t.is_active,
            })
            .collect(),
        last_refresh: state.last_refresh as u64,
        total_count: state.total_count,
    }
}

#[command]
pub fn feed_refresh(
    app: tauri::AppHandle,
    engine: State<'_, FeedEngine>,
    request: Option<FeedRefreshRequest>,
) -> FeedStateResponse {
    let mut feed = engine
        .0
        .lock()
        .expect("FeedEngine mutex poisoned in feed_refresh");
    let state = feed.refresh();
    let mut response = convert_to_response(state);

    // Apply tag filter if specified
    if let Some(ref req) = request {
        if let Some(ref tag_filter) = req.tag_filter {
            let filtered_ids: Vec<String> = feed
                .state()
                .items
                .iter()
                .filter(|i| i.tags.iter().any(|t| t == tag_filter))
                .map(|i| i.id.clone())
                .collect();
            response.items.retain(|i| filtered_ids.contains(&i.id));
            response.total_count = response.items.len();
        }

        if let Some(ref search_query) = req.search_query {
            if !search_query.is_empty() {
                let search_ids: Vec<String> = feed
                    .search(search_query)
                    .iter()
                    .map(|i| i.id.clone())
                    .collect();
                response.items.retain(|i| search_ids.contains(&i.id));
                response.total_count = response.items.len();
            }
        }
    }

    let _ = app.emit("feed-update", &response);
    response
}

#[command]
pub fn feed_search(engine: State<'_, FeedEngine>, query: String) -> Vec<FeedItemResponse> {
    let feed = engine
        .0
        .lock()
        .expect("FeedEngine mutex poisoned in feed_search");
    feed.search(&query)
        .iter()
        .map(|item| convert_item(item))
        .collect()
}

#[command]
pub fn feed_insight(engine: State<'_, FeedEngine>, item_id: String) -> String {
    let feed = engine
        .0
        .lock()
        .expect("FeedEngine mutex poisoned in feed_insight");
    if let Some(item) = feed.state().items.iter().find(|i| i.id == item_id) {
        feed.generate_insight(item)
    } else {
        "No insight available".to_string()
    }
}

#[command]
pub fn feed_timeline_summary(engine: State<'_, FeedEngine>, timeline_id: String) -> String {
    let feed = engine
        .0
        .lock()
        .expect("FeedEngine mutex poisoned in feed_timeline_summary");
    if let Some(tl) = feed.state().timelines.iter().find(|t| t.id == timeline_id) {
        feed.summarize_timeline(tl)
    } else {
        "Timeline not found".to_string()
    }
}

#[command]
pub fn feed_stream_start(
    app: tauri::AppHandle,
    engine: State<'_, FeedEngine>,
    stream_state: State<'_, FeedStreamState>,
) -> Result<String, String> {
    let stream_id = format!(
        "feed-stream-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );

    let stop_flag = Arc::new(AtomicBool::new(false));

    {
        let mut streams = stream_state
            .0
            .lock()
            .map_err(|e| format!("FeedStreamState mutex poisoned: {}", e))?;
        streams.insert(stream_id.clone(), stop_flag.clone());
    }

    let engine_inner = engine.0.clone();

    std::thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(30));

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let mut feed = match engine_inner.lock() {
                Ok(f) => f,
                Err(_) => continue,
            };
            let state = feed.refresh();
            let response = convert_to_response(state);
            drop(feed);

            let _ = app.emit("feed-update", &response);
        }
    });

    Ok(stream_id)
}

#[command]
pub fn feed_stream_stop(
    stream_state: State<'_, FeedStreamState>,
    stream_id: String,
) -> Result<(), String> {
    let mut streams = stream_state
        .0
        .lock()
        .map_err(|e| format!("FeedStreamState mutex poisoned: {}", e))?;

    if let Some(stop_flag) = streams.remove(&stream_id) {
        stop_flag.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err(format!("Stream not found: {}", stream_id))
    }
}

#[command]
pub fn feed_get_tags(engine: State<'_, FeedEngine>) -> Vec<TagResponse> {
    let feed = engine
        .0
        .lock()
        .expect("FeedEngine mutex poisoned in feed_get_tags");
    feed.state()
        .tags
        .iter()
        .map(|t| TagResponse {
            name: t.name.clone(),
            count: t.count as u64,
            is_active: t.is_active,
        })
        .collect()
}
