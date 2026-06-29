use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::connector::VideoInfo;
use crate::neotrix::nt_act_social::filter::{FilterContext, ScoredMoment, SelfControlledFilter};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

#[derive(Clone)]
pub struct IosAppState {
    pub agent: Arc<RwLock<SelfIteratingBrain>>,
    pub filter: Arc<RwLock<SelfControlledFilter>>,
    pub auth: Arc<SocialAuth>,
}

#[derive(Serialize)]
pub struct MomentResponse {
    pub id: String,
    pub title: String,
    pub url: String,
    pub platform: String,
    pub thumbnail: Option<String>,
    pub author: Option<String>,
    pub score: f64,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub duration_secs: Option<u64>,
}

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub moment_id: String,
    pub liked: bool,
    pub keywords: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct MomentQuery {
    pub platforms: Option<String>,
    pub keywords: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Serialize)]
pub struct SocialStatus {
    pub platform: String,
    pub logged_in: bool,
}

#[derive(Deserialize)]
pub struct SocialLoginRequest {
    pub platform: String,
    pub token: String,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Deserialize)]
pub struct ScoreRequest {
    pub moments: Vec<VideoInfoSubmission>,
}

#[derive(Deserialize, Clone)]
pub struct VideoInfoSubmission {
    pub id: String,
    pub title: String,
    pub description: String,
    pub platform: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub duration_secs: Option<u64>,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub author: Option<String>,
    pub published_at: Option<i64>,
}

impl From<VideoInfoSubmission> for VideoInfo {
    fn from(s: VideoInfoSubmission) -> Self {
        VideoInfo {
            id: s.id,
            title: s.title,
            description: s.description,
            platform: crate::neotrix::nt_act_social::connector::Platform::YouTube,
            url: s.url,
            thumbnail_url: s.thumbnail_url,
            duration_secs: s.duration_secs,
            view_count: s.view_count,
            like_count: s.like_count,
            comment_count: s.comment_count,
            author: s.author,
            published_at: s
                .published_at
                .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()),
        }
    }
}

fn moment_to_response(m: &ScoredMoment) -> MomentResponse {
    MomentResponse {
        id: m.moment.id.clone(),
        title: m.moment.title.clone(),
        url: m.moment.url.clone(),
        platform: m.moment.platform.to_string(),
        thumbnail: m.moment.thumbnail_url.clone(),
        author: m.moment.author.clone(),
        score: m.score,
        view_count: m.moment.view_count,
        like_count: m.moment.like_count,
        duration_secs: m.moment.duration_secs,
    }
}

async fn score_handler(
    State(state): State<IosAppState>,
    Json(req): Json<ScoreRequest>,
) -> Result<Json<Vec<MomentResponse>>, Infallible> {
    let filter = state.filter.read().await;
    let context = FilterContext::default();
    let moments: Vec<VideoInfo> = req.moments.into_iter().map(|s| s.into()).collect();
    let scored = filter.filter_and_rank(moments, &context);
    let diversified = filter.apply_diversity_boost(scored);
    let response: Vec<MomentResponse> = diversified.iter().map(moment_to_response).collect();
    Ok(Json(response))
}

async fn score_stream_handler(
    State(state): State<IosAppState>,
    Json(req): Json<ScoreRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let filter = state.filter.read().await;
    let context = FilterContext::default();
    let moments: Vec<VideoInfo> = req.moments.into_iter().map(|s| s.into()).collect();
    let scored = filter.filter_and_rank(moments, &context);
    let diversified = filter.apply_diversity_boost(scored);

    let stream = stream::iter(diversified.into_iter().enumerate().map(|(i, m)| {
        let json = serde_json::to_string(&moment_to_response(&m)).unwrap_or_default();
        Ok(Event::default().data(json).id(i.to_string()))
    }));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(15)),
    )
}

async fn feedback_handler(
    State(state): State<IosAppState>,
    Json(req): Json<FeedbackRequest>,
) -> Result<Json<&'static str>, Infallible> {
    let filter = state.filter.write().await;
    filter.record_feedback(&req.moment_id, req.liked);
    if let Some(kws) = &req.keywords {
        for kw in kws {
            filter.record_keyword_feedback(kw, req.liked);
        }
    }
    Ok(Json("ok"))
}

async fn social_status_handler(State(state): State<IosAppState>) -> Json<Vec<SocialStatus>> {
    let platforms = [
        "youtube",
        "tiktok",
        "douyin",
        "instagram",
        "twitter",
        "reddit",
        "bilibili",
    ];
    let statuses: Vec<SocialStatus> = platforms
        .iter()
        .map(|p| SocialStatus {
            platform: p.to_string(),
            logged_in: state.auth.is_token_valid(p),
        })
        .collect();
    Json(statuses)
}

async fn social_login_handler(
    State(state): State<IosAppState>,
    Json(req): Json<SocialLoginRequest>,
) -> Result<Json<&'static str>, Infallible> {
    state.auth.set_token(
        &req.platform,
        crate::neotrix::nt_act_social::PlatformTokens {
            access_token: req.token,
            refresh_token: req.refresh_token,
            expires_at: Some(chrono::Utc::now().timestamp() + 86400 * 30),
            scope: None,
        },
    );
    Ok(Json("ok"))
}

async fn chat_handler(
    State(state): State<IosAppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, Infallible> {
    let mut agent = state.agent.write().await;
    let reply = if let Some(ref mut engine) = agent.reasoning_engine {
        engine
            .reason(&req.message)
            .unwrap_or_else(|e| format!("error: {}", e))
    } else {
        format!("received: {}", req.message)
    };
    Ok(Json(ChatResponse { reply }))
}

async fn chat_stream_handler(
    State(state): State<IosAppState>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut agent = state.agent.write().await;
    let reply = if let Some(ref mut engine) = agent.reasoning_engine {
        engine
            .reason(&req.message)
            .unwrap_or_else(|e| format!("error: {}", e))
    } else {
        format!("received: {}", req.message)
    };

    let events: Vec<Result<Event, Infallible>> = reply
        .chars()
        .map(|c| Ok(Event::default().data(c.to_string())))
        .collect();
    Sse::new(stream::iter(events)).keep_alive(
        axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(30)),
    )
}

async fn filter_config_handler(
    State(state): State<IosAppState>,
) -> Result<Json<serde_json::Value>, Infallible> {
    let filter = state.filter.read().await;
    let cfg = filter.config();
    let keywords = filter.top_keywords(20);
    let json = serde_json::json!({
        "weights": {
            "relevance": cfg.relevance_weight,
            "recency": cfg.recency_weight,
            "engagement": cfg.engagement_weight,
            "diversity": cfg.diversity_weight,
            "quality": cfg.quality_weight,
            "novelty": cfg.novelty_weight,
        },
        "min_score": cfg.min_score,
        "max_results": cfg.max_results,
        "top_keywords": keywords,
    });
    Ok(Json(json))
}

pub fn ios_routes() -> Router<IosAppState> {
    Router::new()
        .route("/api/v1/moments/score", post(score_handler))
        .route("/api/v1/moments/score-stream", post(score_stream_handler))
        .route("/api/v1/moments/feedback", post(feedback_handler))
        .route("/api/v1/social/status", get(social_status_handler))
        .route("/api/v1/social/login", post(social_login_handler))
        .route("/api/v1/chat", post(chat_handler))
        .route("/api/v1/chat/stream", post(chat_stream_handler))
        .route("/api/v1/filter/config", get(filter_config_handler))
}
