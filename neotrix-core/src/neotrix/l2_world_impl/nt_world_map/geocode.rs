use crate::neotrix::l2_world_impl::nt_world_map::types::GeoPoint;

#[derive(Debug, Clone)]
pub struct GeocodingResult {
    pub point: GeoPoint,
    pub label: String,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ReverseGeocodingResult {
    pub label: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub confidence: f64,
}

pub trait Geocoder: Send + Sync {
    fn geocode(&self, query: &str) -> Vec<GeocodingResult>;
    fn reverse(&self, point: &GeoPoint) -> Option<ReverseGeocodingResult>;
    fn name(&self) -> &'static str;
}

pub struct BuiltinGeocoder {
    known_places: Vec<(GeoPoint, &'static str, &'static str)>,
}

impl Default for BuiltinGeocoder {
    fn default() -> Self {
        Self {
            known_places: vec![
                (GeoPoint::new(40.7128, -74.0060), "New York City", "US"),
                (GeoPoint::new(48.8566, 2.3522), "Paris", "FR"),
                (GeoPoint::new(51.5074, -0.1278), "London", "GB"),
                (GeoPoint::new(35.6762, 139.6503), "Tokyo", "JP"),
                (GeoPoint::new(39.9042, 116.4074), "Beijing", "CN"),
                (GeoPoint::new(31.2304, 121.4737), "Shanghai", "CN"),
                (GeoPoint::new(22.3193, 114.1694), "Hong Kong", "CN"),
                (GeoPoint::new(25.0343, 121.5645), "Taipei", "TW"),
                (GeoPoint::new(37.5665, 126.9780), "Seoul", "KR"),
                (GeoPoint::new(55.7558, 37.6173), "Moscow", "RU"),
                (GeoPoint::new(52.5200, 13.4050), "Berlin", "DE"),
                (GeoPoint::new(48.1351, 11.5820), "Munich", "DE"),
                (GeoPoint::new(41.9028, 12.4964), "Rome", "IT"),
                (GeoPoint::new(40.4168, -3.7038), "Madrid", "ES"),
                (GeoPoint::new(-33.8688, 151.2093), "Sydney", "AU"),
                (GeoPoint::new(-23.5505, -46.6333), "Sao Paulo", "BR"),
                (GeoPoint::new(19.0760, 72.8777), "Mumbai", "IN"),
                (GeoPoint::new(28.6139, 77.2090), "New Delhi", "IN"),
                (GeoPoint::new(1.3521, 103.8198), "Singapore", "SG"),
                (GeoPoint::new(-1.2864, 36.8172), "Nairobi", "KE"),
            ],
        }
    }
}

impl Geocoder for BuiltinGeocoder {
    fn geocode(&self, query: &str) -> Vec<GeocodingResult> {
        let q = query.to_lowercase();
        self.known_places.iter().filter_map(|(point, name, country)| {
            let label = format!("{}, {}", name, country);
            if name.to_lowercase().contains(&q) || country.to_lowercase().contains(&q) {
                Some(GeocodingResult {
                    point: *point,
                    label,
                    confidence: if name.to_lowercase() == q { 1.0 } else { 0.7 },
                    source: "builtin".to_string(),
                })
            } else { None }
        }).collect()
    }

    fn reverse(&self, point: &GeoPoint) -> Option<ReverseGeocodingResult> {
        let mut best: Option<(f64, usize)> = None;
        for (i, entry) in self.known_places.iter().enumerate() {
            let dist = point.distance_haversine(&entry.0);
            let threshold = 50_000.0;
            if dist < threshold {
                let score = 1.0 - dist / threshold;
                match best {
                    Some((ref best_score, _)) if score <= *best_score => {}
                    _ => best = Some((score, i)),
                }
            }
        }
        best.map(|(confidence, i)| {
            let (_, name, country) = &self.known_places[i];
            ReverseGeocodingResult {
                label: name.to_string(),
                country: Some(country.to_string()),
                region: None,
                city: Some(name.to_string()),
                confidence,
            }
        })
    }

    fn name(&self) -> &'static str { "builtin" }
}

pub struct CompositeGeocoder {
    geocoders: Vec<Box<dyn Geocoder>>,
}

impl Default for CompositeGeocoder {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeGeocoder {
    pub fn new() -> Self {
        Self { geocoders: vec![Box::new(BuiltinGeocoder::default())] }
    }

    pub fn add(&mut self, geocoder: Box<dyn Geocoder>) {
        self.geocoders.push(geocoder);
    }
}

impl Geocoder for CompositeGeocoder {
    fn geocode(&self, query: &str) -> Vec<GeocodingResult> {
        let mut results = Vec::new();
        for g in &self.geocoders {
            results.extend(g.geocode(query));
        }
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    fn reverse(&self, point: &GeoPoint) -> Option<ReverseGeocodingResult> {
        for g in &self.geocoders {
            if let Some(r) = g.reverse(point) {
                return Some(r);
            }
        }
        None
    }

    fn name(&self) -> &'static str { "composite" }
}

pub fn format_coordinate(point: &GeoPoint) -> String {
    let ns = if point.lat >= 0.0 { "N" } else { "S" };
    let ew = if point.lng >= 0.0 { "E" } else { "W" };
    format!("{:.4}°{} {:.4}°{}", point.lat.abs(), ns, point.lng.abs(), ew)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_geocode_city() {
        let g = BuiltinGeocoder::default();
        let results = g.geocode("Tokyo");
        assert!(!results.is_empty());
        assert!(results[0].label.contains("Tokyo"));
        assert!((results[0].point.lat - 35.6762).abs() < 0.01);
    }

    #[test]
    fn test_builtin_geocode_country() {
        let g = BuiltinGeocoder::default();
        let results = g.geocode("FR");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.label.contains("Paris")));
    }

    #[test]
    fn test_builtin_reverse() {
        let g = BuiltinGeocoder::default();
        let result = g.reverse(&GeoPoint::new(48.85, 2.35));
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().city.as_deref(), Some("Paris"));
    }

    #[test]
    fn test_reverse_no_match() {
        let g = BuiltinGeocoder::default();
        let result = g.reverse(&GeoPoint::new(0.0, 0.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_composite_geocoder() {
        let g = CompositeGeocoder::new();
        let results = g.geocode("London");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_format_coordinate() {
        let f = format_coordinate(&GeoPoint::new(40.7128, -74.0060));
        assert!(f.contains("N"));
        assert!(f.contains("W"));
        assert!(f.contains("40.7128"));
    }

    #[test]
    fn test_empty_query() {
        let g = BuiltinGeocoder::default();
        let results = g.geocode("Atlantis");
        assert!(results.is_empty());
    }
}
