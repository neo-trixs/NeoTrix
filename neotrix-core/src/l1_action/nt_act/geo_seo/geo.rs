//! 地理位置类型
//!
//! 定义通用的地理位置、区域等类型，支持所有需要地理位置能力的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 地理位置 — 跨域通用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// 纬度
    pub lat: f64,
    /// 经度
    pub lng: f64,
    /// 国家代码 (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// 省/州
    pub region: Option<String>,
    /// 城市
    pub city: Option<String>,
    /// 邮政编码
    pub postal_code: Option<String>,
    /// 时区
    pub timezone: Option<String>,
}

impl GeoLocation {
    /// 创建新位置
    pub fn new(lat: f64, lng: f64) -> Self {
        Self {
            lat,
            lng,
            country: None,
            region: None,
            city: None,
            postal_code: None,
            timezone: None,
        }
    }

    /// 设置国家
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// 设置城市
    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    /// 计算两点间距离 (Haversine公式, 单位: 公里)
    pub fn distance_to(&self, other: &GeoLocation) -> f64 {
        let r = 6371.0; // 地球半径 (km)
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lng = (other.lng - self.lng).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos() * other.lat.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    /// 是否在指定半径内
    pub fn is_within_radius(&self, other: &GeoLocation, radius_km: f64) -> bool {
        self.distance_to(other) <= radius_km
    }
}

/// 地理区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoRegion {
    /// 区域名称
    pub name: String,
    /// 区域代码
    pub code: String,
    /// 区域类型
    pub region_type: RegionType,
    /// 中心点
    pub center: Option<GeoLocation>,
    /// 边界框 (min_lat, max_lat, min_lng, max_lng)
    pub bounding_box: Option<(f64, f64, f64, f64)>,
}

/// 区域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionType {
    Continent,
    Country,
    State,
    City,
    District,
    Custom,
}

/// 位置类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationType {
    /// 固定地址
    Fixed,
    /// 移动位置
    Mobile,
    /// 虚拟位置
    Virtual,
    /// 未知
    Unknown,
}

impl std::fmt::Display for GeoLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(city) = &self.city {
            write!(f, "{}", city)?;
        }
        if let Some(country) = &self.country {
            write!(f, ", {}", country)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_location() {
        let loc = GeoLocation::new(39.9042, 116.4074)
            .with_country("CN")
            .with_city("Beijing");
        
        assert_eq!(loc.lat, 39.9042);
        assert_eq!(loc.lng, 116.4074);
        assert_eq!(loc.country, Some("CN".into()));
        assert_eq!(loc.city, Some("Beijing".into()));
    }

    #[test]
    fn test_distance() {
        let beijing = GeoLocation::new(39.9042, 116.4074);
        let shanghai = GeoLocation::new(31.2304, 121.4737);
        
        let dist = beijing.distance_to(&shanghai);
        assert!((1000.0..1200.0).contains(&dist)); // ~1068 km
    }

    #[test]
    fn test_within_radius() {
        let center = GeoLocation::new(39.9042, 116.4074);
        let nearby = GeoLocation::new(39.9100, 116.4100);
        let far = GeoLocation::new(31.2304, 121.4737);
        
        assert!(center.is_within_radius(&nearby, 10.0));
        assert!(!center.is_within_radius(&far, 10.0));
    }
}
