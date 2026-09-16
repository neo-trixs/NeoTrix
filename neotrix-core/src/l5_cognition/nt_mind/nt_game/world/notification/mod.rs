#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    Achievement,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: f64,
    pub elapsed: f64,
    pub dismissed: bool,
}

impl Notification {
    pub fn new(title: &str, message: &str, notification_type: NotificationType) -> Self {
        Self {
            id: 0,
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            duration: 3.0,
            elapsed: 0.0,
            dismissed: false,
        }
    }

    pub fn with_duration(mut self, d: f64) -> Self {
        self.duration = d;
        self
    }

    pub fn tick(&mut self, dt: f64) {
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.dismissed = true;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.dismissed || self.elapsed >= self.duration
    }
}

pub struct NotificationQueue {
    notifications: Vec<Notification>,
    next_id: u64,
    max_visible: usize,
}

impl NotificationQueue {
    pub fn new(max_visible: usize) -> Self {
        Self {
            notifications: Vec::new(),
            next_id: 1,
            max_visible,
        }
    }

    pub fn push(&mut self, mut n: Notification) {
        n.id = self.next_id;
        self.next_id += 1;
        self.notifications.push(n);
    }

    pub fn tick(&mut self, dt: f64) {
        for n in &mut self.notifications {
            n.tick(dt);
        }
        self.notifications.retain(|n| !n.is_expired());
    }

    pub fn visible(&self) -> Vec<&Notification> {
        self.notifications.iter().take(self.max_visible).collect()
    }

    pub fn dismiss(&mut self, id: u64) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.dismissed = true;
        }
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    pub fn count(&self) -> usize {
        self.notifications.len()
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new(5)
    }
}
