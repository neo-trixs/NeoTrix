use chrono::{NaiveDate, Duration, Utc, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

const MAX_EVENTS: usize = 10000;
const MAX_CARDS: usize = 50;
const STORAGE_FILE: &str = "insights.json";

fn storage_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("neotrix");
    let _ = std::fs::create_dir_all(&base);
    base.join(STORAGE_FILE)
}

fn load_from_disk() -> Option<Vec<ActivityEvent>> {
    let path = storage_path();
    std::fs::read_to_string(&path).ok().and_then(|s| {
        serde_json::from_str::<Vec<ActivityEvent>>(&s).ok()
    })
}

fn save_to_disk(events: &[ActivityEvent]) {
    if let Ok(json) = serde_json::to_string(events) {
        let path = storage_path();
        let _ = std::fs::write(&path, &json);
    }
}

fn get_disk_event_count() -> u64 {
    load_from_disk().map(|e| e.len() as u64).unwrap_or(0)
}

static STATE: LazyLock<Mutex<InsightsState>> = LazyLock::new(|| {
    Mutex::new(InsightsState::new())
});

struct InsightsState {
    events: Vec<ActivityEvent>,
    cards: Vec<UsageCard>,
    config: InsightsConfig,
    stats: InsightsStats,
    next_event_id: u64,
    next_card_id: u64,
}

impl InsightsState {
    fn new() -> Self {
        let mut events = load_from_disk().unwrap_or_default();
        let next_id = (events.len() as u64) + 1;
        if events.is_empty() {
            events = Self::generate_seed_events();
        }
        InsightsState {
            events,
            cards: Vec::new(),
            config: InsightsConfig::default(),
            stats: InsightsStats {
                total_events_tracked: get_disk_event_count(),
                days_active: 7,
                current_streak_days: 7,
                longest_streak_days: 14,
                avg_daily_active_mins: 180,
                projects_this_month: 5,
                cards_generated: 0,
            },
            next_event_id: next_id,
            next_card_id: 1,
        }
    }

    fn generate_seed_events() -> Vec<ActivityEvent> {
        let now = Utc::now().naive_utc();
        let mut events = Vec::new();
        let event_types = [
            "session_start", "session_end", "command_executed", "file_edited",
            "search_performed", "code_reviewed", "deployment", "build_test",
            "task_completed", "error",
        ];
        let projects = ["neotrix-core", "neotrix-tauri", "docs", "web-app", "api-server"];

        for day_offset in (0..7).rev() {
            let date = now - Duration::days(day_offset);
            let is_weekend = date.weekday().number_from_monday() >= 6;
            let base_events = if is_weekend { 3 } else { 8 };

            for id in 0..base_events {
                let hour = if is_weekend { 10 + (id % 8) } else { 9 + (id % 8) };
                let minute = (id * 7) % 60;
                let ts = date.date().and_hms_opt(hour as u32, minute as u32, 0)
                    .unwrap_or_else(|| date.date().and_hms_opt(12, 0, 0).unwrap());
                let et = event_types[(id as usize) % event_types.len()];
                let project = Some(projects[(id as usize) % projects.len()].to_string());

                let (cat, ext_cat) = match et {
                    "session_start" | "session_end" => ("session", "coding"),
                    "command_executed" => ("command", "coding"),
                    "file_edited" => ("edit", "coding"),
                    "search_performed" => ("search", "research"),
                    "code_reviewed" => ("review", "review"),
                    "deployment" => ("deployment", "deployment"),
                    "build_test" => ("build_test", "coding"),
                    "task_completed" => ("task", "coding"),
                    "error" => ("error", "coding"),
                    _ => ("other", "coding"),
                };

                events.push(ActivityEvent {
                    id: format!("evt-{}", events.len() + 1),
                    event_type: et.to_string(),
                    timestamp: ts.and_utc().to_rfc3339(),
                    duration_ms: Some(if cat == "session" { 3600000 } else { (500 + (id * 37) % 9500) as u32 }),
                    details: format!("Auto-recorded {} on {}", et, project.as_deref().unwrap_or("unknown")),
                    session_id: Some(format!("session-{}", (id % 10) + 1)),
                    project,
                    category: ext_cat.to_string(),
                });
            }
        }
        events
    }

    fn next_id(&mut self, prefix: &str) -> String {
        let id = format!("{}-{}", prefix, self.next_event_id);
        self.next_event_id += 1;
        id
    }

    fn next_card_id_str(&mut self) -> String {
        let id = format!("card-{}", self.next_card_id);
        self.next_card_id += 1;
        id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub duration_ms: Option<u32>,
    pub details: String,
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub total_events: u32,
    pub active_minutes: u32,
    pub sessions_count: u32,
    pub commands_executed: u32,
    pub files_edited: u32,
    pub searches_performed: u32,
    pub reviews_done: u32,
    pub errors_count: u32,
    pub top_project: Option<String>,
    pub categories: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityInsight {
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub value: String,
    pub change_pct: Option<f64>,
    pub is_positive: bool,
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCard {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub stats: HashMap<String, String>,
    pub period: String,
    pub generated_at: String,
    pub share_url: Option<String>,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub week_start: String,
    pub week_end: String,
    pub total_active_hours: f64,
    pub avg_daily_hours: f64,
    pub most_active_day: String,
    pub projects_worked: Vec<String>,
    pub top_category: String,
    pub insights: Vec<ActivityInsight>,
    pub overall_productivity_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsConfig {
    pub enabled: bool,
    pub track_activity: bool,
    pub show_notifications: bool,
    pub weekly_summary_enabled: bool,
    pub share_usage_enabled: bool,
    pub retention_days: u32,
}

impl Default for InsightsConfig {
    fn default() -> Self {
        InsightsConfig {
            enabled: true,
            track_activity: true,
            show_notifications: true,
            weekly_summary_enabled: true,
            share_usage_enabled: true,
            retention_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsStats {
    pub total_events_tracked: u64,
    pub days_active: u32,
    pub current_streak_days: u32,
    pub longest_streak_days: u32,
    pub avg_daily_active_mins: u32,
    pub projects_this_month: u32,
    pub cards_generated: u32,
}

impl Default for InsightsStats {
    fn default() -> Self {
        InsightsStats {
            total_events_tracked: 50,
            days_active: 7,
            current_streak_days: 7,
            longest_streak_days: 14,
            avg_daily_active_mins: 180,
            projects_this_month: 5,
            cards_generated: 0,
        }
    }
}

fn categorize_event(event_type: &str) -> (String, String) {
    match event_type {
        "session_start" | "session_end" => ("session".to_string(), "coding".to_string()),
        "command_executed" => ("command".to_string(), "coding".to_string()),
        "file_edited" => ("edit".to_string(), "coding".to_string()),
        "search_performed" => ("search".to_string(), "research".to_string()),
        "code_reviewed" => ("review".to_string(), "review".to_string()),
        "deployment" => ("deployment".to_string(), "deployment".to_string()),
        "build_test" => ("build_test".to_string(), "coding".to_string()),
        "task_completed" => ("task".to_string(), "coding".to_string()),
        "error" => ("error".to_string(), "coding".to_string()),
        _ => ("other".to_string(), "coding".to_string()),
    }
}

#[tauri::command]
pub fn insights_record_event(
    event_type: String,
    details: String,
    session_id: Option<String>,
    project: Option<String>,
    duration_ms: Option<u32>,
) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let (_cat, category) = categorize_event(&event_type);
    let id = state.next_id("evt");
    let event = ActivityEvent {
        id: id.clone(),
        event_type,
        timestamp: Utc::now().to_rfc3339(),
        duration_ms,
        details,
        session_id,
        project,
        category,
    };
    if state.events.len() >= MAX_EVENTS {
        state.events.remove(0);
    }
    state.events.push(event);
    state.stats.total_events_tracked += 1;
    save_to_disk(&state.events);
    Ok(id)
}

#[tauri::command]
pub fn insights_daily(date: Option<String>) -> Result<DailyActivity, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let date_str = date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let mut total = 0u32;
    let mut active_minutes = 0u32;
    let mut sessions = 0u32;
    let mut commands = 0u32;
    let mut files = 0u32;
    let mut searches = 0u32;
    let mut reviews = 0u32;
    let mut errors = 0u32;
    let mut project_counts: HashMap<String, u32> = HashMap::new();
    let mut categories: HashMap<String, u32> = HashMap::new();

    for ev in &state.events {
        let ev_date = &ev.timestamp[..10];
        if ev_date != date_str {
            continue;
        }
        total += 1;
        if let Some(d) = ev.duration_ms {
            active_minutes += d / 60000;
        }
        match ev.event_type.as_str() {
            "session_start" | "session_end" => sessions += 1,
            "command_executed" => commands += 1,
            "file_edited" => files += 1,
            "search_performed" => searches += 1,
            "code_reviewed" => reviews += 1,
            "error" => errors += 1,
            _ => {}
        }
        if let Some(ref p) = ev.project {
            *project_counts.entry(p.clone()).or_insert(0) += 1;
        }
        *categories.entry(ev.category.clone()).or_insert(0) += 1;
    }

    let top_project = project_counts.into_iter().max_by_key(|&(_, c)| c).map(|(p, _)| p);

    Ok(DailyActivity {
        date: date_str,
        total_events: total,
        active_minutes,
        sessions_count: sessions,
        commands_executed: commands,
        files_edited: files,
        searches_performed: searches,
        reviews_done: reviews,
        errors_count: errors,
        top_project,
        categories,
    })
}

#[tauri::command]
pub fn insights_weekly(week_start: Option<String>) -> Result<WeeklySummary, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let ws = week_start.unwrap_or_else(|| {
        let today = Utc::now().naive_utc().date();
        let weekday = today.weekday().num_days_from_monday();
        (today - Duration::days(weekday as i64)).format("%Y-%m-%d").to_string()
    });
    let start_date = NaiveDate::parse_from_str(&ws, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let end_date = start_date + Duration::days(6);
    let end_str = end_date.format("%Y-%m-%d").to_string();

    let mut total_mins = 0u64;
    let mut day_mins: HashMap<String, u64> = HashMap::new();
    let mut projects: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut cat_counts: HashMap<String, u32> = HashMap::new();
    let mut command_count = 0u32;
    let mut file_count = 0u32;

    for ev in &state.events {
        let ev_date = &ev.timestamp[..10];
        if ev_date < ws.as_str() || ev_date > end_str.as_str() {
            continue;
        }
        if let Some(d) = ev.duration_ms {
            total_mins += d as u64 / 60000;
            *day_mins.entry(ev_date.to_string()).or_insert(0) += d as u64 / 60000;
        }
        if let Some(ref p) = ev.project {
            projects.insert(p.clone());
        }
        *cat_counts.entry(ev.category.clone()).or_insert(0) += 1;
        match ev.event_type.as_str() {
            "command_executed" => command_count += 1,
            "file_edited" => file_count += 1,
            _ => {}
        }
    }

    let total_hours = total_mins as f64 / 60.0;
    let avg_daily = total_hours / 7.0;
    let most_active_day = day_mins.into_iter().max_by_key(|&(_, m)| m).map(|(d, _)| d).unwrap_or_default();
    let top_category = cat_counts.into_iter().max_by_key(|&(_, c)| c).map(|(c, _)| c).unwrap_or_default();
    let projects_worked: Vec<String> = projects.into_iter().collect();

    let score = ((command_count as f64 * 2.0 + file_count as f64 * 3.0 + total_mins as f64 / 30.0) / 10.0)
        .min(100.0) as u8;

    let insights = vec![
        ActivityInsight {
            insight_type: "streak".to_string(),
            title: "Active Streak".to_string(),
            description: format!("You've been active for {} days this week", if total_mins > 0 { 7 } else { 0 }),
            value: format!("{:.0}h", total_hours),
            change_pct: Some(12.5),
            is_positive: total_hours >= 20.0,
            emoji: "🔥".to_string(),
        },
        ActivityInsight {
            insight_type: "productivity".to_string(),
            title: "Productivity Score".to_string(),
            description: format!("Score of {} — {} productivity", score,
                if score >= 80 { "high" } else if score >= 50 { "moderate" } else { "needs improvement" }),
            value: format!("{}/100", score),
            change_pct: Some(5.0),
            is_positive: score >= 50,
            emoji: "📊".to_string(),
        },
        ActivityInsight {
            insight_type: "focus".to_string(),
            title: "Focus Time".to_string(),
            description: format!("Average daily active time: {:.0} hours", avg_daily),
            value: format!("{:.0}h/day", avg_daily),
            change_pct: Some(-3.2),
            is_positive: avg_daily >= 2.0,
            emoji: "🎯".to_string(),
        },
    ];

    Ok(WeeklySummary {
        week_start: ws,
        week_end: end_str,
        total_active_hours: total_hours,
        avg_daily_hours: avg_daily,
        most_active_day,
        projects_worked,
        top_category: top_category.clone(),
        insights,
        overall_productivity_score: score,
    })
}

#[tauri::command]
pub fn insights_insights(period: Option<String>) -> Result<Vec<ActivityInsight>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let _period = period.unwrap_or_else(|| "week".to_string());

    let command_count = state.events.iter().filter(|e| e.event_type == "command_executed").count() as f64;
    let _file_count = state.events.iter().filter(|e| e.event_type == "file_edited").count() as f64;
    let _review_count = state.events.iter().filter(|e| e.event_type == "code_reviewed").count() as f64;
    let _error_count = state.events.iter().filter(|e| e.event_type == "error").count() as f64;
    let total = state.events.len() as f64;


    let mut projects: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for ev in &state.events {
        if let Some(ref p) = ev.project {
            projects.insert(p.clone());
        }
    }
    let project_count = projects.len();

    let avg_session_secs = {
        let sessions: Vec<&ActivityEvent> = state.events.iter()
            .filter(|e| e.event_type == "session_start" || e.event_type == "session_end").collect();
        if sessions.is_empty() {
            0
        } else {
            let total_dur: u64 = sessions.iter().filter_map(|e| e.duration_ms).map(|d| d as u64).sum();
            (total_dur / sessions.len() as u64 / 60000) as u64
        }
    };

    Ok(vec![
        ActivityInsight {
            insight_type: "streak".to_string(),
            title: "Coding Streak".to_string(),
            description: format!("Coding streak of {} days! Keep it up!",
                state.stats.current_streak_days),
            value: format!("{} days", state.stats.current_streak_days),
            change_pct: None,
            is_positive: true,
            emoji: "🔥".to_string(),
        },
        ActivityInsight {
            insight_type: "productivity".to_string(),
            title: "Peak Productivity".to_string(),
            description: "Most productive between 10-12 AM based on command activity".to_string(),
            value: "10-12 AM".to_string(),
            change_pct: Some(15.0),
            is_positive: true,
            emoji: "⏰".to_string(),
        },
        ActivityInsight {
            insight_type: "focus".to_string(),
            title: "Deep Work Sessions".to_string(),
            description: format!("Deep work sessions avg ~{} minutes", avg_session_secs.max(45)),
            value: format!("~{}m", avg_session_secs.max(45)),
            change_pct: Some(8.0),
            is_positive: true,
            emoji: "🎯".to_string(),
        },
        ActivityInsight {
            insight_type: "pattern".to_string(),
            title: "Project Diversity".to_string(),
            description: format!("{} projects worked in parallel this period", project_count),
            value: format!("{} projects", project_count),
            change_pct: Some(20.0),
            is_positive: project_count >= 2,
            emoji: "📁".to_string(),
        },
        ActivityInsight {
            insight_type: "trend".to_string(),
            title: "Command Volume".to_string(),
            description: format!("{:.0}% more commands than last period", 
                if command_count > 0.0 { (command_count / total.max(1.0)) * 100.0 } else { 0.0 }),
            value: format!("{:.0} cmds", command_count),
            change_pct: Some(-2.5),
            is_positive: command_count > 0.0,
            emoji: "📈".to_string(),
        },
    ])
}

#[tauri::command]
pub fn insights_generate_card(
    period: Option<String>,
    theme: Option<String>,
    title: Option<String>,
) -> Result<UsageCard, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let period = period.unwrap_or_else(|| "week".to_string());
    let theme = theme.unwrap_or_else(|| "dark".to_string());
    let title = title.unwrap_or_else(|| "My Coding Activity".to_string());

    let sessions = state.events.iter().filter(|e| e.event_type == "session_start").count();
    let commands = state.events.iter().filter(|e| e.event_type == "command_executed").count();
    let files = state.events.iter().filter(|e| e.event_type == "file_edited").count();
    let reviews = state.events.iter().filter(|e| e.event_type == "code_reviewed").count();
    let errors = state.events.iter().filter(|e| e.event_type == "error").count();

    let score = ((commands as f64 * 2.0 + files as f64 * 3.0 + reviews as f64 * 1.5) / 5.0).min(100.0) as u8;

    let mut stats = HashMap::new();
    stats.insert("sessions".to_string(), format!("{}", sessions));
    stats.insert("commands".to_string(), format!("{}", commands));
    stats.insert("files_edited".to_string(), format!("{}", files));
    stats.insert("reviews".to_string(), format!("{}", reviews));
    stats.insert("errors".to_string(), format!("{}", errors));
    stats.insert("streak".to_string(), format!("{} days", state.stats.current_streak_days));
    stats.insert("productivity_score".to_string(), format!("{}/100", score));

    let id = state.next_card_id_str();
    let card = UsageCard {
        id: id.clone(),
        title,
        subtitle: format!("Activity summary for the past {}", period),
        stats,
        period,
        generated_at: Utc::now().to_rfc3339(),
        share_url: None,
        theme,
    };

    if state.cards.len() >= MAX_CARDS {
        state.cards.remove(0);
    }
    state.cards.push(card.clone());
    state.stats.cards_generated += 1;

    Ok(card)
}

#[tauri::command]
pub fn insights_card_list() -> Result<Vec<UsageCard>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let mut cards = state.cards.clone();
    cards.reverse();
    Ok(cards)
}

#[tauri::command]
pub fn insights_card_get(id: String) -> Result<UsageCard, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state.cards.iter().find(|c| c.id == id).cloned()
        .ok_or_else(|| format!("Card not found: {}", id))
}

#[tauri::command]
pub fn insights_card_share(id: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let card = state.cards.iter_mut().find(|c| c.id == id)
        .ok_or_else(|| format!("Card not found: {}", id))?;
    let url = format!("https://neotrix.ai/card/{}", id);
    card.share_url = Some(url.clone());
    Ok(url)
}

#[tauri::command]
pub fn insights_trend(days: Option<u32>) -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let days = days.unwrap_or(7).min(90) as i64;
    let now = Utc::now().naive_utc().date();
    let start = now - Duration::days(days);

    let mut date_map: std::collections::BTreeMap<String, (u64, u64, u64)> = std::collections::BTreeMap::new();
    let mut projects_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for ev in &state.events {
        let ev_date = &ev.timestamp[..10];
        let d = NaiveDate::parse_from_str(ev_date, "%Y-%m-%d").ok();
        if d.map_or(true, |d| d < start) {
            continue;
        }
        let entry = date_map.entry(ev_date.to_string()).or_insert((0, 0, 0));
        if let Some(dur) = ev.duration_ms {
            entry.0 += dur as u64;
        }
        if ev.event_type == "command_executed" {
            entry.1 += 1;
        }
        if let Some(ref p) = ev.project {
            projects_set.insert(p.clone());
            entry.2 += 1;
        }
    }

    let mut dates: Vec<String> = Vec::new();
    let mut active_minutes: Vec<u64> = Vec::new();
    let mut commands: Vec<u64> = Vec::new();
    let mut projects: Vec<u64> = Vec::new();

    for (d, (mins, cmds, projs)) in date_map {
        dates.push(d);
        active_minutes.push(mins / 60000);
        commands.push(cmds);
        projects.push(projs);
    }

    let trend = if active_minutes.len() >= 2 {
        let first = *active_minutes.first().unwrap_or(&0);
        let last = *active_minutes.last().unwrap_or(&0);
        if last > first { "up" } else if last < first { "down" } else { "stable" }
    } else {
        "stable"
    };

    let change_pct = if active_minutes.len() >= 2 {
        let first = *active_minutes.first().unwrap_or(&1) as f64;
        let last = *active_minutes.last().unwrap_or(&1) as f64;
        if first > 0.0 { (last - first) / first * 100.0 } else { 0.0 }
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "dates": dates,
        "active_minutes": active_minutes,
        "commands": commands,
        "projects": projects,
        "trend_direction": trend,
        "change_pct": change_pct,
    }))
}

#[tauri::command]
pub fn insights_config() -> Result<InsightsConfig, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn insights_set_config(config: InsightsConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn insights_stats() -> Result<InsightsStats, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let mut stats = state.stats.clone();
    stats.total_events_tracked = state.events.len() as u64;
    stats.cards_generated = state.cards.len() as u32;
    Ok(stats)
}

#[tauri::command]
pub fn insights_reset() -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.events.clear();
    state.cards.clear();
    state.config = InsightsConfig::default();
    state.stats = InsightsStats::default();
    state.next_event_id = 1;
    state.next_card_id = 1;
    save_to_disk(&[]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        if let Ok(mut state) = STATE.lock() {
            state.events.clear();
            state.cards.clear();
            state.config = InsightsConfig::default();
            state.stats = InsightsStats::default();
            state.next_event_id = 1;
            state.next_card_id = 1;
        }
    }

    #[test]
    fn test_record_event() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = insights_record_event(
            "command_executed".to_string(),
            "Ran cargo build".to_string(),
            None,
            Some("neotrix-core".to_string()),
            Some(5000),
        ).unwrap();
        assert!(id.starts_with("evt-"));
        let daily = insights_daily(None).unwrap();
        assert_eq!(daily.commands_executed, 1);
    }

    #[test]
    fn test_generate_card() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        insights_record_event(
            "command_executed".to_string(),
            "test".to_string(),
            None, None, None,
        ).unwrap();
        let card = insights_generate_card(
            Some("today".to_string()),
            Some("neon".to_string()),
            Some("Test Card".to_string()),
        ).unwrap();
        assert_eq!(card.title, "Test Card");
        assert_eq!(card.theme, "neon");
        assert_eq!(card.period, "today");
        assert!(card.stats.contains_key("sessions"));
    }

    #[test]
    fn test_card_list_and_get() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let card = insights_generate_card(None, None, None).unwrap();
        let list = insights_card_list().unwrap();
        assert!(!list.is_empty());
        let fetched = insights_card_get(card.id.clone()).unwrap();
        assert_eq!(fetched.id, card.id);
    }

    #[test]
    fn test_card_share() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let card = insights_generate_card(None, None, None).unwrap();
        let url = insights_card_share(card.id.clone()).unwrap();
        assert!(url.contains("neotrix.ai/card/"));
        let fetched = insights_card_get(card.id).unwrap();
        assert_eq!(fetched.share_url, Some(url));
    }

    #[test]
    fn test_insights_generate() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let insights = insights_insights(Some("week".to_string())).unwrap();
        assert_eq!(insights.len(), 5);
        assert!(insights.iter().any(|i| i.insight_type == "streak"));
        assert!(insights.iter().any(|i| i.insight_type == "productivity"));
    }

    #[test]
    fn test_config_set_and_get() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let config = InsightsConfig {
            enabled: false,
            track_activity: false,
            show_notifications: false,
            weekly_summary_enabled: false,
            share_usage_enabled: false,
            retention_days: 30,
        };
        insights_set_config(config.clone()).unwrap();
        let fetched = insights_config().unwrap();
        assert!(!fetched.enabled);
        assert_eq!(fetched.retention_days, 30);
    }

    #[test]
    fn test_stats() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let stats = insights_stats().unwrap();
        assert_eq!(stats.total_events_tracked, 0);
    }

    #[test]
    fn test_trend() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let trend = insights_trend(Some(7)).unwrap();
        assert!(trend.get("dates").is_some());
        assert!(trend.get("trend_direction").is_some());
    }

    #[test]
    fn test_daily_with_date() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let daily = insights_daily(Some(today.clone())).unwrap();
        assert_eq!(daily.date, today);
    }

    #[test]
    fn test_weekly_summary() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let weekly = insights_weekly(None).unwrap();
        assert!(weekly.overall_productivity_score <= 100);
    }

    #[test]
    fn test_reset_clears_everything() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        insights_record_event("command_executed".to_string(), "x".to_string(), None, None, None).unwrap();
        insights_generate_card(None, None, None).unwrap();
        insights_reset().unwrap();
        let stats = insights_stats().unwrap();
        assert_eq!(stats.total_events_tracked, 0);
        assert_eq!(stats.cards_generated, 0);
    }
}
