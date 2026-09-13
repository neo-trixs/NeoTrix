use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Game State
// ---------------------------------------------------------------------------

/// Top-level game states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Loading,
    Title,
    Creating,
    Playing,
    Paused,
    Dialogue,
    Combat,
    Cutscene,
}

impl GameState {
    /// Whether this state accepts update ticks
    pub fn updates_paused(&self) -> bool {
        matches!(self, Self::Loading | Self::Title | Self::Paused | Self::Dialogue | Self::Cutscene)
    }
}

// ---------------------------------------------------------------------------
// State Stack (overlay system)
// ---------------------------------------------------------------------------

/// A stack entry: the game state plus an opaque overlay identifier.
#[derive(Debug, Clone)]
pub struct StackEntry {
    pub state: GameState,
    pub overlay_id: Option<String>,
}

/// State stack allows pushing overlays (e.g. pause menu on top of Playing).
#[derive(Debug)]
pub struct StateStack {
    entries: VecDeque<StackEntry>,
}

impl StateStack {
    pub fn new(initial: GameState) -> Self {
        let mut entries = VecDeque::new();
        entries.push_back(StackEntry { state: initial, overlay_id: None });
        Self { entries }
    }

    /// Current (topmost) state.
    pub fn current(&self) -> GameState {
        self.entries.back().map_or(GameState::Loading, |e| e.state)
    }

    /// Push a new state on top.
    pub fn push(&mut self, state: GameState) {
        self.entries.push_back(StackEntry { state, overlay_id: None });
    }

    /// Push with an overlay id.
    pub fn push_overlay(&mut self, state: GameState, id: &str) {
        self.entries.push_back(StackEntry { state, overlay_id: Some(id.to_string()) });
    }

    /// Pop the topmost state. Returns the popped entry, or None if stack has 1 element.
    pub fn pop(&mut self) -> Option<StackEntry> {
        if self.entries.len() > 1 {
            self.entries.pop_back()
        } else {
            None
        }
    }

    /// Replace the topmost state.
    pub fn replace(&mut self, state: GameState) {
        if let Some(entry) = self.entries.back_mut() {
            entry.state = state;
        }
    }

    /// How many states are in the stack.
    pub fn depth(&self) -> usize {
        self.entries.len()
    }

    /// Check if a specific overlay id is present.
    pub fn has_overlay(&self, id: &str) -> bool {
        self.entries.iter().any(|e| e.overlay_id.as_deref() == Some(id))
    }

    /// Remove an overlay by id.
    pub fn remove_overlay(&mut self, id: &str) -> bool {
        let len = self.entries.len();
        self.entries.retain(|e| e.overlay_id.as_deref() != Some(id));
        self.entries.len() < len
    }

    /// Iterate states from bottom to top.
    pub fn iter(&self) -> impl Iterator<Item = &StackEntry> {
        self.entries.iter()
    }
}

impl Default for StateStack {
    fn default() -> Self {
        Self::new(GameState::Loading)
    }
}

// ---------------------------------------------------------------------------
// Game Event Bus (lightweight, for inter-system comms within core loop)
// ---------------------------------------------------------------------------

/// Lightweight event payload for the core game loop.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    StateChange { from: GameState, to: GameState },
    Quit,
    FocusLost,
    FocusGained,
    Resize { width: u32, height: u32 },
    Custom(String),
}

/// Simple ring-buffer event bus for core loop events.
#[derive(Debug)]
pub struct CoreEventBus {
    events: VecDeque<CoreEvent>,
    capacity: usize,
}

impl CoreEventBus {
    pub fn new(capacity: usize) -> Self {
        Self { events: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, event: CoreEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn drain(&mut self) -> Vec<CoreEvent> {
        self.events.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for CoreEventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

// ---------------------------------------------------------------------------
// Fixed Timestep Game Loop
// ---------------------------------------------------------------------------

/// Configuration for the fixed-timestep loop.
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// Target updates per second (e.g. 60).
    pub tick_rate: f64,
    /// Maximum frames to process per tick to avoid spiral of death.
    pub max_frame_skip: u32,
    /// Whether to run in headless mode (no render).
    pub headless: bool,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self { tick_rate: 60.0, max_frame_skip: 5, headless: false }
    }
}

/// Accumulated timing for the fixed-timestep loop.
#[derive(Debug)]
pub struct LoopTimer {
    pub dt: f64,
    pub elapsed: f64,
    pub frame_count: u64,
    pub fps: f64,
    last_fps_report: Instant,
    fps_frame_count: u64,
    accumulator: f64,
}

impl LoopTimer {
    pub fn new(tick_rate: f64) -> Self {
        Self {
            dt: 1.0 / tick_rate,
            elapsed: 0.0,
            frame_count: 0,
            fps: 0.0,
            last_fps_report: Instant::now(),
            fps_frame_count: 0,
            accumulator: 0.0,
        }
    }

    /// Call once per frame with wall-clock delta. Returns how many fixed
    /// ticks should be executed.
    pub fn tick(&mut self, wall_dt: Duration, tick_rate: f64, max_frame_skip: u32) -> u32 {
        let delta = wall_dt.as_secs_f64();
        self.accumulator += delta;
        let fixed_dt = 1.0 / tick_rate;
        let mut ticks = 0u32;

        while self.accumulator >= fixed_dt && ticks < max_frame_skip {
            ticks += 1;
            self.accumulator -= fixed_dt;
            self.elapsed += fixed_dt;
            self.frame_count += 1;
            self.fps_frame_count += 1;
        }

        // FPS counter
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_report).as_secs_f64();
        if elapsed >= 1.0 {
            self.fps = self.fps_frame_count as f64 / elapsed;
            self.fps_frame_count = 0;
            self.last_fps_report = now;
        }

        ticks
    }
}

impl Default for LoopTimer {
    fn default() -> Self {
        Self::new(60.0)
    }
}

// ---------------------------------------------------------------------------
// GameEngine — top-level driver
// ---------------------------------------------------------------------------

/// Callback types for the engine lifecycle.
pub trait GameLoopCallbacks {
    /// Called once at startup.
    fn on_init(&mut self);
    /// Called once per fixed tick.
    fn on_tick(&mut self, dt: f64);
    /// Called once per frame (variable rate).
    fn on_render(&mut self, alpha: f64);
    /// Called on state change.
    fn on_state_change(&mut self, from: GameState, to: GameState);
    /// Called on shutdown.
    fn on_shutdown(&mut self);
}

/// The game engine driving state + loop + events.
pub struct GameEngine {
    pub state_stack: StateStack,
    pub event_bus: CoreEventBus,
    pub timer: LoopTimer,
    pub config: LoopConfig,
    running: bool,
}

impl GameEngine {
    pub fn new(config: LoopConfig) -> Self {
        let initial = GameState::Loading;
        let timer = LoopTimer::new(config.tick_rate);
        Self {
            state_stack: StateStack::new(initial),
            event_bus: CoreEventBus::new(256),
            timer,
            config,
            running: false,
        }
    }

    /// Current game state.
    pub fn state(&self) -> GameState {
        self.state_stack.current()
    }

    /// Transition to a new state (pops any overlays, replaces top).
    pub fn set_state(&mut self, new_state: GameState) {
        let old = self.state_stack.current();
        if old != new_state {
            self.state_stack.replace(new_state);
            self.event_bus.push(CoreEvent::StateChange { from: old, to: new_state });
        }
    }

    /// Push an overlay state.
    pub fn push_state(&mut self, state: GameState) {
        let old = self.state_stack.current();
        self.state_stack.push(state);
        self.event_bus.push(CoreEvent::StateChange { from: old, to: state });
    }

    /// Pop the overlay, returning to the previous state.
    pub fn pop_state(&mut self) -> Option<GameState> {
        let old = self.state_stack.current();
        self.state_stack.pop().map(|e| {
            let new = self.state_stack.current();
            if old != new {
                self.event_bus.push(CoreEvent::StateChange { from: old, to: new });
            }
            e.state
        })
    }

    /// Request quit.
    pub fn quit(&mut self) {
        self.running = false;
        self.event_bus.push(CoreEvent::Quit);
    }

    /// Run the engine with the given callbacks (blocking).
    pub fn run<C: GameLoopCallbacks>(&mut self, callbacks: &mut C) {
        self.running = true;
        callbacks.on_init();

        let tick_rate = self.config.tick_rate;
        let max_skip = self.config.max_frame_skip;
        let mut last_time = Instant::now();

        while self.running {
            let wall_dt = last_time.elapsed();
            last_time = Instant::now();

            // Drain core events
            let events = self.event_bus.drain();
            for event in &events {
                match event {
                    CoreEvent::Quit => {
                        self.running = false;
                        break;
                    }
                    CoreEvent::StateChange { from, to } => {
                        callbacks.on_state_change(*from, *to);
                    }
                    _ => {}
                }
            }
            if !self.running {
                break;
            }

            // Fixed timestep update
            let ticks = self.timer.tick(wall_dt, tick_rate, max_skip);
            if !self.state().updates_paused() {
                for _ in 0..ticks {
                    callbacks.on_tick(self.timer.dt);
                }
            }

            // Variable-rate render
            let alpha = self.timer.accumulator / (1.0 / tick_rate);
            callbacks.on_render(alpha);
        }

        callbacks.on_shutdown();
    }

    /// Quick tick (non-blocking) — useful for tests / embedded use.
    pub fn tick_once<C: GameLoopCallbacks>(&mut self, callbacks: &mut C, wall_dt: Duration) {
        let ticks = self.timer.tick(wall_dt, self.config.tick_rate, self.config.max_frame_skip);
        for _ in 0..ticks {
            callbacks.on_tick(self.timer.dt);
        }
        let alpha = self.timer.accumulator / (1.0 / self.config.tick_rate);
        callbacks.on_render(alpha);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_updates_paused() {
        assert!(!GameState::Playing.updates_paused());
        assert!(GameState::Paused.updates_paused());
        assert!(GameState::Dialogue.updates_paused());
        assert!(GameState::Loading.updates_paused());
    }

    #[test]
    fn test_state_stack_push_pop() {
        let mut stack = StateStack::new(GameState::Loading);
        assert_eq!(stack.current(), GameState::Loading);
        stack.push(GameState::Playing);
        assert_eq!(stack.current(), GameState::Playing);
        stack.push(GameState::Paused);
        assert_eq!(stack.current(), GameState::Paused);
        let popped = stack.pop();
        assert_eq!(popped.unwrap().state, GameState::Paused);
        assert_eq!(stack.current(), GameState::Playing);
    }

    #[test]
    fn test_state_stack_overlay() {
        let mut stack = StateStack::new(GameState::Playing);
        stack.push_overlay(GameState::Dialogue, "npc_talk");
        assert!(stack.has_overlay("npc_talk"));
        assert_eq!(stack.current(), GameState::Dialogue);
        stack.remove_overlay("npc_talk");
        assert!(!stack.has_overlay("npc_talk"));
        assert_eq!(stack.current(), GameState::Playing);
    }

    #[test]
    fn test_state_stack_cannot_pop_last() {
        let mut stack = StateStack::new(GameState::Loading);
        assert!(stack.pop().is_none());
        assert_eq!(stack.current(), GameState::Loading);
    }

    #[test]
    fn test_core_event_bus() {
        let mut bus = CoreEventBus::new(4);
        bus.push(CoreEvent::Quit);
        bus.push(CoreEvent::FocusLost);
        assert_eq!(bus.len(), 2);
        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_core_event_bus_overflow() {
        let mut bus = CoreEventBus::new(2);
        bus.push(CoreEvent::Quit);
        bus.push(CoreEvent::FocusLost);
        bus.push(CoreEvent::FocusGained); // should evict Quit
        assert_eq!(bus.len(), 2);
        let events = bus.drain();
        assert!(matches!(&events[0], CoreEvent::FocusLost));
    }

    #[test]
    fn test_loop_timer_tick() {
        let mut timer = LoopTimer::new(60.0);
        let ticks = timer.tick(Duration::from_secs_f64(1.0 / 60.0), 60.0, 5);
        assert_eq!(ticks, 1);
        let ticks = timer.tick(Duration::from_secs_f64(1.0), 60.0, 5);
        assert!(ticks <= 5); // max_frame_skip cap
    }

    #[test]
    fn test_game_engine_set_state() {
        let mut engine = GameEngine::new(LoopConfig::default());
        assert_eq!(engine.state(), GameState::Loading);
        engine.set_state(GameState::Playing);
        assert_eq!(engine.state(), GameState::Playing);
        // Process events
        let events = engine.event_bus.drain();
        assert!(!events.is_empty());
    }
}
