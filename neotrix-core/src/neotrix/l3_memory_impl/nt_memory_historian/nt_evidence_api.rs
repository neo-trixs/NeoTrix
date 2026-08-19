use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

use super::nt_evidence_store::EvidenceStore;
use super::nt_evidence_types::EvidenceRecord;

use super::nt_evidence_hypothesis::{
    HypothesisNetwork, SubjectiveOpinion,
};
use super::nt_evidence_credibility::{
    CredibilityAggregator, SourceCredibility, SourceTier,
};
use super::nt_evidence_temporal::{
    AnachronismDetector, TemporalEvidenceTracker, TimelineReconstructor,
};

#[derive(Clone)]
pub struct EvidenceApiState {
    pub store: Arc<Mutex<EvidenceStore>>,
    pub hypothesis_network: Arc<Mutex<HypothesisNetwork>>,
    pub credibility_aggregator: Arc<Mutex<CredibilityAggregator>>,
    pub temporal_tracker: Arc<Mutex<TemporalEvidenceTracker>>,
    pub anachronism_detector: Arc<Mutex<AnachronismDetector>>,
    pub timeline: Arc<Mutex<TimelineReconstructor>>,
}

impl EvidenceApiState {
    pub fn try_open_default() -> Option<Self> {
        EvidenceStore::try_open_default().map(|s| Self {
            store: Arc::new(Mutex::new(s)),
            hypothesis_network: Arc::new(Mutex::new(HypothesisNetwork::new())),
            credibility_aggregator: Arc::new(Mutex::new(CredibilityAggregator::new())),
            temporal_tracker: Arc::new(Mutex::new(TemporalEvidenceTracker::new())),
            anachronism_detector: Arc::new(Mutex::new(AnachronismDetector::new())),
            timeline: Arc::new(Mutex::new(TimelineReconstructor::new())),
        })
    }
}

pub fn build_ewhr_router(state: EvidenceApiState) -> Router {
    Router::new()
        // CRUD
        .route("/api/ewhr/list", get(list_handler))
        .route("/api/ewhr/get/{id}", get(get_handler))
        .route("/api/ewhr/add", post(add_handler))
        .route("/api/ewhr/delete/{id}", post(delete_handler))
        .route("/api/ewhr/calibrate", post(calibrate_handler))
        .route("/api/ewhr/stats", get(stats_handler))
        // Hypothesis endpoints
        .route("/api/ewhr/hypothesis/propose", post(propose_hypothesis_handler))
        .route("/api/ewhr/hypothesis/{id}", get(get_hypothesis_handler))
        .route("/api/ewhr/hypothesis/{id}/update", post(update_hypothesis_handler))
        .route("/api/ewhr/hypothesis/list", get(list_hypotheses_handler))
        .route("/api/ewhr/hypothesis/strongest", get(strongest_hypothesis_handler))
        // Credibility endpoints
        .route("/api/ewhr/credibility/add", post(add_credibility_handler))
        .route("/api/ewhr/credibility/aggregate", get(aggregate_credibility_handler))
        .route("/api/ewhr/credibility/geometric", get(geometric_credibility_handler))
        .route("/api/ewhr/credibility/diversity", get(diversity_credibility_handler))
        .route("/api/ewhr/credibility/trust_propagation", post(trust_propagation_handler))
        // Temporal endpoints
        .route("/api/ewhr/temporal/record", post(record_temporal_handler))
        .route("/api/ewhr/temporal/trend/{id}", get(trend_handler))
        .route("/api/ewhr/temporal/consistency/{id}", get(consistency_handler))
        .route("/api/ewhr/temporal/timeline/add", post(add_timeline_event_handler))
        .route("/api/ewhr/temporal/timeline/sorted", get(sorted_timeline_handler))
        .route("/api/ewhr/temporal/timeline/gaps", get(timeline_gaps_handler))
        .route("/api/ewhr/temporal/anachronism/check", post(check_anachronism_handler))
        .route("/api/ewhr/temporal/anachronism/register", post(register_entity_handler))
        .route("/api/ewhr/temporal/allen/{a_start}/{a_end}/{b_start}/{b_end}", get(allen_handler))
        // Opinion endpoints
        .route("/api/ewhr/opinion/fuse", post(fuse_opinion_handler))
        .route("/api/ewhr/opinion/average", post(average_opinion_handler))
        .route("/api/ewhr/audit/{id}", get(audit_history_handler))
        .route("/api/ewhr/audit/recent/{n}", get(audit_recent_handler))
        .with_state(state)
}

fn json_ok<T: Serialize>(v: T) -> Json<serde_json::Value> {
    Json(json!(v))
}

fn json_err(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({"error": msg})))
}

#[derive(Deserialize)]
struct ProposeHypothesisParams {
    id: String,
    title: String,
    description: String,
    prior: f64,
}

#[derive(Deserialize)]
struct UpdateHypothesisParams {
    evidence_confidence: f64,
    supports_hypothesis: bool,
    evidence_strength: f64,
}

#[derive(Deserialize)]
struct AddCredibilityParams {
    tier: String,
    review_status: String,
    author_reputation: f64,
    institutional_backing: f64,
    citation_count: u64,
    temporal_proximity: f64,
    independence_score: f64,
    cross_validation_count: u32,
}

#[derive(Deserialize)]
struct RecordTemporalParams {
    evidence_id: String,
    timestamp: i64,
    value: f64,
}

#[derive(Deserialize)]
struct AddTimelineEventParams {
    id: String,
    description: String,
    timestamp: i64,
    evidence_ids: Vec<String>,
    confidence: f64,
}

#[derive(Deserialize)]
struct CheckAnachronismParams {
    claim_timestamp: i64,
    entity: String,
}

#[derive(Deserialize)]
struct RegisterEntityParams {
    entity: String,
    start: i64,
    end: i64,
}

#[derive(Deserialize)]
struct FuseOpinionParams {
    belief_a: f64,
    disbelief_a: f64,
    uncertainty_a: f64,
    base_rate_a: f64,
    belief_b: f64,
    disbelief_b: f64,
    uncertainty_b: f64,
    base_rate_b: f64,
}

#[derive(Deserialize)]
struct AverageOpinionParams {
    opinions: Vec<FuseOpinionParams>,
}

#[derive(Deserialize)]
struct TrustPropagationParams {
    citations: Vec<(usize, usize)>,
}

async fn list_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.store.lock() {
        Ok(store) => match store.list_evidence() {
            Ok(records) => json_ok(records),
            Err(e) => json_ok(json!({"error": e, "records": []})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e), "records": []})),
    }
}

async fn get_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.store.lock() {
        Ok(store) => match store.get_evidence(&id) {
            Ok(Some(record)) => {
                let tier = record.tier();
                let suff = record.sufficiency();
                json_ok(json!({
                    "found": true,
                    "record": record,
                    "tier": tier.label(),
                    "tier_color": tier.color(),
                    // scansci-pi 证据优先门: 显式声明证据充分性, 不静默置信度
                    "sufficient_evidence": suff.is_sufficient(),
                    "insufficient_reasons": suff.reasons(),
                }))
            }
            Ok(None) => json_ok(json!({"found": false})),
            Err(e) => json_ok(json!({"error": e})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn add_handler(
    State(state): State<EvidenceApiState>,
    Json(record): Json<EvidenceRecord>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.store.lock() {
        Ok(store) => match store.store_evidence(&record) {
            Ok(()) => {
                let tier = record.tier();
                (StatusCode::OK, Json(json!({"status": "ok", "id": record.id, "tier": tier.label()})))
            }
            Err(e) => json_err(&e),
        },
        Err(e) => json_err(&format!("lock: {}", e)),
    }
}

async fn delete_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.store.lock() {
        Ok(store) => match store.delete_evidence(&id) {
            Ok(()) => json_ok(json!({"status": "deleted", "id": id})),
            Err(e) => json_ok(json!({"error": e})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn calibrate_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.store.lock() {
        Ok(store) => match store.calibrate() {
            Ok(result) => json_ok(result),
            Err(e) => json_ok(json!({"error": e})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn stats_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.store.lock() {
        Ok(store) => match store.stats() {
            Ok(stats) => json_ok(stats),
            Err(e) => json_ok(json!({"error": e})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn propose_hypothesis_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<ProposeHypothesisParams>,
) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(mut net) => {
            let h = net.propose_hypothesis(&params.id, &params.title, &params.description, params.prior);
            json_ok(json!({
                "status": "proposed",
                "id": h.id,
                "prior": h.prior_probability,
                "posterior": h.posterior_probability,
                "status_label": h.status.label(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn get_hypothesis_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(net) => match net.get_hypothesis(&id) {
            Some(h) => json_ok(json!({
                "found": true,
                "hypothesis": {
                    "id": h.id,
                    "title": h.title,
                    "description": h.description,
                    "status": h.status.label(),
                    "prior": h.prior_probability,
                    "posterior": h.posterior_probability,
                    "supporting_weight": h.supporting_weight,
                    "refuting_weight": h.refuting_weight,
                    "evidence_ids": h.evidence_ids,
                    "summary": h.bayes_factor_summary(),
                }
            })),
            None => json_ok(json!({"found": false})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn update_hypothesis_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
    Json(params): Json<UpdateHypothesisParams>,
) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(mut net) => match net.get_hypothesis_mut(&id) {
            Some(h) => {
                h.update_with_evidence(params.evidence_confidence, params.supports_hypothesis, params.evidence_strength);
                json_ok(json!({
                    "status": "updated",
                    "id": h.id,
                    "posterior": h.posterior_probability,
                    "supporting_weight": h.supporting_weight,
                    "refuting_weight": h.refuting_weight,
                    "status_label": h.status.label(),
                    "summary": h.bayes_factor_summary(),
                }))
            }
            None => json_ok(json!({"error": "hypothesis not found"})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn list_hypotheses_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(net) => json_ok(json!({
            "count": net.hypotheses.len(),
            "hypotheses": net.hypotheses.iter().map(|h| json!({
                "id": h.id,
                "title": h.title,
                "status": h.status.label(),
                "posterior": h.posterior_probability,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn strongest_hypothesis_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(net) => {
            let supported = net.find_strongest_supported();
            let refuted = net.find_strongest_refuted();
            json_ok(json!({
                "strongest_supported": supported.map(|h| json!({
                    "id": h.id,
                    "title": h.title,
                    "posterior": h.posterior_probability,
                })),
                "strongest_refuted": refuted.map(|h| json!({
                    "id": h.id,
                    "title": h.title,
                    "posterior": h.posterior_probability,
                })),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn add_credibility_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<AddCredibilityParams>,
) -> Json<serde_json::Value> {
    let tier = match params.tier.to_lowercase().as_str() {
        "primary" => SourceTier::Primary,
        "secondary" => SourceTier::Secondary,
        "tertiary" => SourceTier::Tertiary,
        "hearsay" => SourceTier::Hearsay,
        "anonymous" => SourceTier::Anonymous,
        other => {
            return json_ok(json!({"error": format!("unknown tier: {}", other)}));
        }
    };
    let review = match params.review_status.to_lowercase().as_str() {
        "peer_reviewed" => super::nt_evidence_credibility::ReviewStatus::PeerReviewed,
        "preprint" => super::nt_evidence_credibility::ReviewStatus::Preprint,
        "conference" => super::nt_evidence_credibility::ReviewStatus::ConferenceProceedings,
        "self_published" => super::nt_evidence_credibility::ReviewStatus::SelfPublished,
        "unreviewed" => super::nt_evidence_credibility::ReviewStatus::Unreviewed,
        other => {
            return json_ok(json!({"error": format!("unknown review_status: {}", other)}));
        }
    };
    let cred = SourceCredibility {
        source_tier: tier,
        review_status: review,
        author_reputation: params.author_reputation.clamp(0.0, 1.0),
        institutional_backing: params.institutional_backing.clamp(0.0, 1.0),
        citation_count: params.citation_count,
        temporal_proximity: params.temporal_proximity.clamp(0.0, 1.0),
        custody_chain: None,
        independence_score: params.independence_score.clamp(0.0, 1.0),
        cross_validation_count: params.cross_validation_count,
    };
    let score = cred.overall_score();
    let tier_label = cred.credibility_tier().to_string();
    match state.credibility_aggregator.lock() {
        Ok(mut agg) => {
            agg.add(cred);
            json_ok(json!({"status": "added", "score": score, "tier_label": tier_label}))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn aggregate_credibility_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.credibility_aggregator.lock() {
        Ok(agg) => {
            let weighted = agg.aggregate_weighted();
            json_ok(json!({
                "score": weighted,
                "n_sources": agg.scores.len(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn geometric_credibility_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.credibility_aggregator.lock() {
        Ok(agg) => {
            let geometric = agg.aggregate_geometric();
            json_ok(json!({
                "geometric": geometric,
                "n_sources": agg.scores.len(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn diversity_credibility_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.credibility_aggregator.lock() {
        Ok(agg) => {
            json_ok(json!({
                "diversity": agg.diversity_score(),
                "n_sources": agg.scores.len(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn trust_propagation_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<TrustPropagationParams>,
) -> Json<serde_json::Value> {
    match state.credibility_aggregator.lock() {
        Ok(agg) => {
            let ranks = agg.propagate_trust(&params.citations);
            json_ok(json!({
                "ranks": ranks,
                "n_sources": agg.scores.len(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn record_temporal_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<RecordTemporalParams>,
) -> Json<serde_json::Value> {
    match state.temporal_tracker.lock() {
        Ok(mut tracker) => {
            tracker.record_value(&params.evidence_id, params.timestamp, params.value);
            let rev = tracker.revision_count.get(&params.evidence_id).copied().unwrap_or(0);
            json_ok(json!({"status": "recorded", "evidence_id": params.evidence_id, "revisions": rev}))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn trend_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.temporal_tracker.lock() {
        Ok(tracker) => match tracker.trend(&id) {
            Some(trend) => json_ok(json!({
                "found": true,
                "trend": {
                    "direction": format!("{:?}", trend.direction),
                    "slope": trend.slope,
                    "volatility": trend.volatility,
                    "recent_trend": format!("{:?}", trend.recent_trend),
                    "n_points": trend.n_points,
                }
            })),
            None => json_ok(json!({"found": false})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn consistency_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.temporal_tracker.lock() {
        Ok(tracker) => match tracker.evidence_consistency(&id) {
            Some(consistency) => json_ok(json!({"found": true, "consistency": consistency})),
            None => json_ok(json!({"found": false})),
        },
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn add_timeline_event_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<AddTimelineEventParams>,
) -> Json<serde_json::Value> {
    match state.timeline.lock() {
        Ok(mut tl) => {
            tl.add_event(&params.id, &params.description, params.timestamp, params.evidence_ids, params.confidence);
            json_ok(json!({"status": "added", "event_id": params.id}))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn sorted_timeline_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.timeline.lock() {
        Ok(tl) => {
            let sorted = tl.sorted_events();
            let span = tl.max_time_span();
            json_ok(json!({
                "events": sorted.iter().map(|e| json!({
                    "id": e.id,
                    "description": e.description,
                    "timestamp": e.timestamp,
                    "confidence": e.confidence,
                })).collect::<Vec<_>>(),
                "time_span": span.map(|(min, max)| json!({"start": min, "end": max, "duration": max - min})),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn timeline_gaps_handler(State(state): State<EvidenceApiState>) -> Json<serde_json::Value> {
    match state.timeline.lock() {
        Ok(tl) => {
            let gaps = tl.detect_gaps();
            json_ok(json!({
                "gaps": gaps.iter().map(|(start, end, duration)| json!({
                    "start": start,
                    "end": end,
                    "duration_seconds": duration,
                    "duration_days": duration / 86400,
                })).collect::<Vec<_>>(),
                "n_gaps": gaps.len(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn check_anachronism_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<CheckAnachronismParams>,
) -> Json<serde_json::Value> {
    match state.anachronism_detector.lock() {
        Ok(det) => {
            let result = det.check_anachronism(params.claim_timestamp, &params.entity);
            json_ok(json!({
                "anachronism": result.is_some(),
                "detail": result,
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn register_entity_handler(
    State(state): State<EvidenceApiState>,
    Json(params): Json<RegisterEntityParams>,
) -> Json<serde_json::Value> {
    match state.anachronism_detector.lock() {
        Ok(mut det) => {
            det.register_entity(&params.entity, params.start, params.end);
            json_ok(json!({"status": "registered", "entity": params.entity}))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn allen_handler(
    Path((a_start, a_end, b_start, b_end)): Path<(i64, i64, i64, i64)>,
) -> Json<serde_json::Value> {
    use super::nt_evidence_temporal::allen_relation;
    let rel = allen_relation(a_start, a_end, b_start, b_end);
    json_ok(json!({
        "relation": format!("{:?}", rel),
        "allen_label": rel.allen_label(),
        "a": [a_start, a_end],
        "b": [b_start, b_end],
    }))
}

async fn fuse_opinion_handler(
    Json(params): Json<FuseOpinionParams>,
) -> Json<serde_json::Value> {
    let a = SubjectiveOpinion::new(
        params.belief_a, params.disbelief_a, params.uncertainty_a, params.base_rate_a,
    );
    let b = SubjectiveOpinion::new(
        params.belief_b, params.disbelief_b, params.uncertainty_b, params.base_rate_b,
    );
    let fused = SubjectiveOpinion::cumulative_fusion(&a, &b);
    json_ok(json!({
        "fused": {
            "belief": fused.belief,
            "disbelief": fused.disbelief,
            "uncertainty": fused.uncertainty,
            "base_rate": fused.base_rate,
            "projected_probability": fused.projected_probability(),
        }
    }))
}

async fn average_opinion_handler(
    Json(params): Json<AverageOpinionParams>,
) -> Json<serde_json::Value> {
    let opinions: Vec<SubjectiveOpinion> = params.opinions.iter().map(|o| {
        SubjectiveOpinion::new(o.belief_a, o.disbelief_a, o.uncertainty_a, o.base_rate_a)
    }).collect();
    let avg = SubjectiveOpinion::averaging_fusion(&opinions);
    json_ok(json!({
        "average": {
            "belief": avg.belief,
            "disbelief": avg.disbelief,
            "uncertainty": avg.uncertainty,
            "base_rate": avg.base_rate,
            "projected_probability": avg.projected_probability(),
        },
        "n_opinions": opinions.len(),
    }))
}

async fn audit_history_handler(
    State(state): State<EvidenceApiState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(net) => {
            let history = net.audit.history(&id);
            json_ok(json!({
                "entity_id": id,
                "entries": history.iter().map(|e| json!({
                    "id": e.id,
                    "timestamp": e.timestamp,
                    "action": e.action,
                    "entity_type": e.entity_type,
                    "field_changed": e.field_changed,
                    "old_value": e.old_value,
                    "new_value": e.new_value,
                    "reason": e.reason,
                    "actor": e.actor,
                })).collect::<Vec<_>>(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}

async fn audit_recent_handler(
    State(state): State<EvidenceApiState>,
    Path(n): Path<usize>,
) -> Json<serde_json::Value> {
    match state.hypothesis_network.lock() {
        Ok(net) => {
            let recent = net.audit.recent(n);
            json_ok(json!({
                "entries": recent.iter().map(|e| json!({
                    "id": e.id,
                    "timestamp": e.timestamp,
                    "action": e.action,
                    "entity_type": e.entity_type,
                    "entity_id": e.entity_id,
                    "field_changed": e.field_changed,
                    "new_value": e.new_value,
                    "actor": e.actor,
                })).collect::<Vec<_>>(),
            }))
        }
        Err(e) => json_ok(json!({"error": format!("lock: {}", e)})),
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
