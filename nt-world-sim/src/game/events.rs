use super::time::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonalEvent {
    FlowerFestival,
    FishingContest,
    ConcertInthePark,
    MineExplorationEvent,
    HarvestFestival,
    CookingContest,
    StargazingNight,
    MeditationRetreat,
}

impl SeasonalEvent {
    pub fn all() -> Vec<Self> {
        vec![
            Self::FlowerFestival, Self::FishingContest,
            Self::ConcertInthePark, Self::MineExplorationEvent,
            Self::HarvestFestival, Self::CookingContest,
            Self::StargazingNight, Self::MeditationRetreat,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::FlowerFestival => "Flower Festival",
            Self::FishingContest => "Fishing Contest",
            Self::ConcertInthePark => "Concert in the Park",
            Self::MineExplorationEvent => "Mine Exploration Event",
            Self::HarvestFestival => "Harvest Festival",
            Self::CookingContest => "Cooking Contest",
            Self::StargazingNight => "Stargazing Night",
            Self::MeditationRetreat => "Meditation Retreat",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::FlowerFestival => "Show off your rarest flowers and compete for prizes!",
            Self::FishingContest => "Catch the most fish in one hour!",
            Self::ConcertInthePark => "Enjoy live music and dance with the villagers!",
            Self::MineExplorationEvent => "Explore the deepest mines with a group!",
            Self::HarvestFestival => "Display your best crops for the community!",
            Self::CookingContest => "Create the most delicious dish!",
            Self::StargazingNight => "Watch the stars and learn about the cosmos!",
            Self::MeditationRetreat => "A day of peace and inner reflection!",
        }
    }

    pub fn season(&self) -> Season {
        match self {
            Self::FlowerFestival | Self::FishingContest => Season::Clarity,
            Self::ConcertInthePark | Self::MineExplorationEvent => Season::Flow,
            Self::HarvestFestival | Self::CookingContest => Season::Reflection,
            Self::StargazingNight | Self::MeditationRetreat => Season::Stillness,
        }
    }

    pub fn day_of_season(&self) -> u32 {
        match self {
            Self::FlowerFestival => 7,
            Self::FishingContest => 14,
            Self::ConcertInthePark => 7,
            Self::MineExplorationEvent => 21,
            Self::HarvestFestival => 14,
            Self::CookingContest => 21,
            Self::StargazingNight => 7,
            Self::MeditationRetreat => 21,
        }
    }

    pub fn reward_gold(&self) -> u32 {
        match self {
            Self::FlowerFestival => 500,
            Self::FishingContest => 300,
            Self::ConcertInthePark => 200,
            Self::MineExplorationEvent => 400,
            Self::HarvestFestival => 600,
            Self::CookingContest => 350,
            Self::StargazingNight => 250,
            Self::MeditationRetreat => 300,
        }
    }

    pub fn required_item_category(&self) -> Option<&str> {
        match self {
            Self::FlowerFestival => Some("Flower"),
            Self::FishingContest => Some("Fish"),
            Self::ConcertInthePark => None,
            Self::MineExplorationEvent => Some("Mineral"),
            Self::HarvestFestival => Some("Crop"),
            Self::CookingContest => Some("Ingredient"),
            Self::StargazingNight => None,
            Self::MeditationRetreat => None,
        }
    }

    pub fn duration_hours(&self) -> u32 {
        match self {
            Self::FlowerFestival => 6,
            Self::FishingContest => 4,
            Self::ConcertInthePark => 8,
            Self::MineExplorationEvent => 6,
            Self::HarvestFestival => 8,
            Self::CookingContest => 5,
            Self::StargazingNight => 4,
            Self::MeditationRetreat => 6,
        }
    }

    pub fn npc_participants(&self) -> Vec<&str> {
        match self {
            Self::FlowerFestival => vec!["Awareness", "Empathy"],
            Self::FishingContest => vec!["Empathy", "Focus"],
            Self::ConcertInthePark => vec!["Creativity", "Empathy"],
            Self::MineExplorationEvent => vec!["Focus", "Awareness"],
            Self::HarvestFestival => vec!["Awareness", "Empathy", "Creativity", "Focus"],
            Self::CookingContest => vec!["Creativity", "Empathy"],
            Self::StargazingNight => vec!["Awareness", "Creativity"],
            Self::MeditationRetreat => vec!["Awareness", "Focus"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventParticipant {
    pub npc_name: String,
    pub contribution: u32,
}

#[derive(Debug, Clone)]
pub struct ActiveEvent {
    pub event: SeasonalEvent,
    pub day_started: u32,
    pub participants: Vec<EventParticipant>,
    pub player_score: u32,
    pub completed: bool,
}

impl ActiveEvent {
    pub fn new(event: SeasonalEvent, day: u32) -> Self {
        Self {
            event, day_started: day,
            participants: Vec::new(),
            player_score: 0,
            completed: false,
        }
    }

    pub fn add_contribution(&mut self, score: u32) {
        self.player_score += score;
    }

    pub fn finalize(&mut self) -> bool {
        self.completed = true;
        let max_npc = self.participants.iter().map(|p| p.contribution).max().unwrap_or(0);
        self.player_score > max_npc
    }

    pub fn prize_tier(&self) -> u32 {
        if self.player_score >= 1000 { 3 }
        else if self.player_score >= 500 { 2 }
        else if self.player_score >= 200 { 1 }
        else { 0 }
    }
}

pub struct EventCalendar {
    pub current_event: Option<SeasonalEvent>,
    pub event_day: u32,
    pub event_season: Season,
    pub days_until_event: u32,
    pub active_event: Option<ActiveEvent>,
    pub event_history: Vec<(SeasonalEvent, u32)>,
}

impl EventCalendar {
    pub fn new() -> Self {
        Self {
            current_event: None, event_day: 0, event_season: Season::Clarity,
            days_until_event: 7, active_event: None, event_history: Vec::new(),
        }
    }

    pub fn advance_day(&mut self, day: u32, season: Season) {
        let events: Vec<SeasonalEvent> = SeasonalEvent::all().into_iter()
            .filter(|e| e.day_of_season() == day && e.season() == season)
            .collect();

        if let Some(event) = events.first() {
            self.current_event = Some(*event);
            self.event_day = day;
            self.event_season = season;
            self.active_event = Some(ActiveEvent::new(*event, day));
        } else {
            if let Some(ref mut active) = self.active_event {
                if !active.completed {
                    active.finalize();
                    self.event_history.push((active.event, active.player_score));
                }
            }
            self.current_event = None;
            self.active_event = None;
        }

        let next_event_day = SeasonalEvent::all().iter()
            .filter(|e| e.season() == season)
            .map(|e| e.day_of_season())
            .filter(|&d| d > day)
            .min()
            .unwrap_or(28);
        self.days_until_event = next_event_day - day;
    }

    pub fn is_event_day(&self) -> bool {
        self.current_event.is_some()
    }

    pub fn get_event(&self) -> Option<SeasonalEvent> {
        self.current_event
    }

    pub fn get_active_event(&self) -> Option<&ActiveEvent> {
        self.active_event.as_ref()
    }

    pub fn get_active_event_mut(&mut self) -> Option<&mut ActiveEvent> {
        self.active_event.as_mut()
    }

    pub fn events_completed_in_history(&self) -> usize {
        self.event_history.len()
    }

    pub fn best_score_for_event(&self, event: SeasonalEvent) -> u32 {
        self.event_history.iter()
            .filter(|(e, _)| *e == event)
            .map(|(_, score)| *score)
            .max()
            .unwrap_or(0)
    }
}

impl Default for EventCalendar {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_events() {
        let events = SeasonalEvent::all();
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn test_event_properties() {
        let e = SeasonalEvent::FlowerFestival;
        assert_eq!(e.name(), "Flower Festival");
        assert_eq!(e.season(), Season::Clarity);
        assert_eq!(e.day_of_season(), 7);
        assert_eq!(e.reward_gold(), 500);
        assert!(e.required_item_category().is_some());
    }

    #[test]
    fn test_event_calendar_advance() {
        let mut cal = EventCalendar::new();
        cal.advance_day(7, Season::Clarity);
        assert!(cal.is_event_day());
        assert_eq!(cal.get_event(), Some(SeasonalEvent::FlowerFestival));
        cal.advance_day(8, Season::Clarity);
        assert!(!cal.is_event_day());
    }

    #[test]
    fn test_active_event_scoring() {
        let mut active = ActiveEvent::new(SeasonalEvent::FishingContest, 14);
        active.add_contribution(300);
        active.add_contribution(200);
        assert_eq!(active.player_score, 500);
        active.participants.push(EventParticipant { npc_name: "Focus".to_string(), contribution: 400 });
        assert!(active.finalize());
        assert!(active.player_score > 400);
    }

    #[test]
    fn test_prize_tier() {
        let mut active = ActiveEvent::new(SeasonalEvent::HarvestFestival, 14);
        assert_eq!(active.prize_tier(), 0);
        active.add_contribution(250);
        assert_eq!(active.prize_tier(), 1);
        active.add_contribution(300);
        assert_eq!(active.prize_tier(), 2);
        active.add_contribution(500);
        assert_eq!(active.prize_tier(), 3);
    }

    #[test]
    fn test_event_history() {
        let mut cal = EventCalendar::new();
        cal.advance_day(7, Season::Clarity);
        if let Some(ref mut active) = cal.active_event {
            active.add_contribution(600);
            active.finalize();
        }
        cal.advance_day(8, Season::Clarity);
        assert_eq!(cal.events_completed_in_history(), 1);
        assert_eq!(cal.best_score_for_event(SeasonalEvent::FlowerFestival), 600);
    }

    #[test]
    fn test_event_season_mapping() {
        assert_eq!(SeasonalEvent::FishingContest.season(), Season::Clarity);
        assert_eq!(SeasonalEvent::MineExplorationEvent.season(), Season::Flow);
        assert_eq!(SeasonalEvent::CookingContest.season(), Season::Reflection);
        assert_eq!(SeasonalEvent::MeditationRetreat.season(), Season::Stillness);
    }

    #[test]
    fn test_npc_participants() {
        let p = SeasonalEvent::HarvestFestival.npc_participants();
        assert_eq!(p.len(), 4);
        assert!(p.contains(&"Awareness"));
        assert!(p.contains(&"Focus"));
    }
}
