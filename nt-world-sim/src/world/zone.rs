#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Farm,
    Town,
    Mine,
    Forest,
    Lake,
    Beach,
    Cave,
    Special,
}

#[derive(Debug, Clone)]
pub struct ZoneConnection {
    pub target_zone: String,
    pub entry_point: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub name: String,
    pub connections: Vec<ZoneConnection>,
}

impl Zone {
    pub fn new(zone_type: ZoneType, position: (u32, u32), size: (u32, u32), name: &str) -> Self {
        Self {
            zone_type,
            position,
            size,
            name: name.to_string(),
            connections: Vec::new(),
        }
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.position.0
            && x < self.position.0 + self.size.0
            && y >= self.position.1
            && y < self.position.1 + self.size.1
    }

    pub fn center(&self) -> (u32, u32) {
        (
            self.position.0 + self.size.0 / 2,
            self.position.1 + self.size.1 / 2,
        )
    }

    pub fn connect_to(&mut self, target: &str, entry: (u32, u32)) {
        self.connections.push(ZoneConnection {
            target_zone: target.to_string(),
            entry_point: entry,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_creation() {
        let zone = Zone::new(ZoneType::Farm, (10, 10), (20, 15), "Test Farm");
        assert_eq!(zone.name, "Test Farm");
        assert_eq!(zone.zone_type, ZoneType::Farm);
        assert_eq!(zone.position, (10, 10));
        assert_eq!(zone.size, (20, 15));
    }

    #[test]
    fn test_zone_contains() {
        let zone = Zone::new(ZoneType::Town, (10, 10), (20, 15), "Town");
        assert!(zone.contains(10, 10));
        assert!(zone.contains(29, 24));
        assert!(!zone.contains(30, 24));
        assert!(!zone.contains(9, 10));
    }

    #[test]
    fn test_zone_center() {
        let zone = Zone::new(ZoneType::Forest, (10, 10), (20, 20), "Forest");
        assert_eq!(zone.center(), (20, 20));
    }

    #[test]
    fn test_zone_connections() {
        let mut zone = Zone::new(ZoneType::Mine, (0, 0), (10, 10), "Mine");
        zone.connect_to("Town", (5, 5));
        zone.connect_to("Forest", (0, 5));
        assert_eq!(zone.connections.len(), 2);
        assert_eq!(zone.connections[0].target_zone, "Town");
        assert_eq!(zone.connections[1].entry_point, (0, 5));
    }
}
