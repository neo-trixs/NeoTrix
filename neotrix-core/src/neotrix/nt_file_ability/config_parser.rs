//! 配置字段解析器 (D2+D3)
//!
//! 将逗号分隔的 "key:value" 格式解析为结构化数据。
//! 通用设计: 不绑定具体业务，支持任意 key-value 对。
//!
//! 示例:
//! - "执行标准:美标,压力:150LB,阀体:Q235" → ConfigFields { exec_std: Some("美标"), pressure: Some("150LB"), valve_body: Some("Q235") }
//! - "材质:球铁；压力PN16,CLASSB,丝扣连接" → ConfigFields { material: Some("球铁"), pressure: Some("PN16"), connection: Some("丝扣连接") }

use std::collections::HashMap;

/// 解析后的配置字段
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigFields {
    pub exec_std: Option<String>,     // 执行标准
    pub pressure: Option<String>,     // 压力
    pub valve_body: Option<String>,   // 阀体
    pub valve_stem: Option<String>,   // 阀杆
    pub valve_plate: Option<String>,  // 阀板
    pub valve_seat: Option<String>,   // 阀座
    pub connection: Option<String>,   // 连接方式
    pub drive: Option<String>,        // 驱动方式
    pub color: Option<String>,        // 颜色
    pub material: Option<String>,     // 材质
    pub temperature: Option<String>,  // 温度
    pub extra: HashMap<String, String>, // 未识别的字段
}

impl ConfigFields {
    /// 解析配置字符串
    pub fn parse(raw: &str) -> Self {
        if raw.trim().is_empty() {
            return Self::default();
        }

        let mut fields = Self::default();

        // 支持两种分隔符: 逗号和分号
        let pairs: Vec<&str> = raw.split(|c| c == ',' || c == '；' || c == ';').collect();

        for pair in pairs {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            // 尝试按冒号分割
            if let Some((key, value)) = pair.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                fields.set_field(key, value);
                continue;
            }

            // 尝试按 "XX:" 格式 (无空格分隔)
            // 例如 "压力PN16" → key="压力", value="PN16"
            if let Some((key, value)) = split_chinese_key_value(pair) {
                fields.set_field(&key, &value);
                continue;
            }

            // 无法分割，作为额外字段存储
            fields.extra.insert(pair.to_string(), String::new());
        }

        fields
    }

    /// 设置字段值
    fn set_field(&mut self, key: &str, value: &str) {
        match key {
            "执行标准" | "标准" => self.exec_std = Some(value.to_string()),
            "压力" => self.pressure = Some(value.to_string()),
            "阀体" | "体" => self.valve_body = Some(value.to_string()),
            "阀杆" | "杆" => self.valve_stem = Some(value.to_string()),
            "阀板" | "板" => self.valve_plate = Some(value.to_string()),
            "阀座" | "座" => self.valve_seat = Some(value.to_string()),
            "连接" | "链接" => self.connection = Some(value.to_string()),
            "驱动" => self.drive = Some(value.to_string()),
            "颜色" => self.color = Some(value.to_string()),
            "材质" => self.material = Some(value.to_string()),
            "温度" => self.temperature = Some(value.to_string()),
            _ => {
                self.extra.insert(key.to_string(), value.to_string());
            }
        }
    }

    /// 是否有内容
    pub fn is_empty(&self) -> bool {
        self.exec_std.is_none()
            && self.pressure.is_none()
            && self.valve_body.is_none()
            && self.valve_stem.is_none()
            && self.valve_plate.is_none()
            && self.valve_seat.is_none()
            && self.connection.is_none()
            && self.drive.is_none()
            && self.color.is_none()
            && self.material.is_none()
            && self.temperature.is_none()
            && self.extra.is_empty()
    }

    /// 转为 HashMap (用于序列化)
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(v) = &self.exec_std {
            map.insert("执行标准".to_string(), v.clone());
        }
        if let Some(v) = &self.pressure {
            map.insert("压力".to_string(), v.clone());
        }
        if let Some(v) = &self.valve_body {
            map.insert("阀体".to_string(), v.clone());
        }
        if let Some(v) = &self.valve_stem {
            map.insert("阀杆".to_string(), v.clone());
        }
        if let Some(v) = &self.valve_plate {
            map.insert("阀板".to_string(), v.clone());
        }
        if let Some(v) = &self.valve_seat {
            map.insert("阀座".to_string(), v.clone());
        }
        if let Some(v) = &self.connection {
            map.insert("连接".to_string(), v.clone());
        }
        if let Some(v) = &self.drive {
            map.insert("驱动".to_string(), v.clone());
        }
        if let Some(v) = &self.color {
            map.insert("颜色".to_string(), v.clone());
        }
        if let Some(v) = &self.material {
            map.insert("材质".to_string(), v.clone());
        }
        if let Some(v) = &self.temperature {
            map.insert("温度".to_string(), v.clone());
        }
        for (k, v) in &self.extra {
            map.insert(k.clone(), v.clone());
        }
        map
    }
}

/// 分割中文 key-value 对 (无冒号分隔)
/// 例如 "压力PN16" → ("压力", "PN16")
fn split_chinese_key_value(s: &str) -> Option<(String, String)> {
    let known_keys = [
        "执行标准", "标准", "压力", "阀体", "阀杆", "阀板", "阀座",
        "连接", "链接", "驱动", "颜色", "材质", "温度",
    ];

    for key in &known_keys {
        if let Some(rest) = s.strip_prefix(key) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some((key.to_string(), rest.to_string()));
            }
        }
    }

    None
}

/// 批量解析多个配置字符串
pub fn parse_configs(raws: &[String]) -> Vec<ConfigFields> {
    raws.iter().map(|r| ConfigFields::parse(r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let config = ConfigFields::parse("执行标准:美标,压力:150LB,阀体:Q235");
        assert_eq!(config.exec_std.as_deref(), Some("美标"));
        assert_eq!(config.pressure.as_deref(), Some("150LB"));
        assert_eq!(config.valve_body.as_deref(), Some("Q235"));
    }

    #[test]
    fn test_parse_with_short_keys() {
        let config = ConfigFields::parse("执行标准:美标,压力:PN16,体:球铁,杆:不锈钢");
        assert_eq!(config.exec_std.as_deref(), Some("美标"));
        assert_eq!(config.pressure.as_deref(), Some("PN16"));
        assert_eq!(config.valve_body.as_deref(), Some("球铁"));
        assert_eq!(config.valve_stem.as_deref(), Some("不锈钢"));
    }

    #[test]
    fn test_parse_semicolon_separator() {
        let config = ConfigFields::parse("材质：球铁；压力PN16");
        assert_eq!(config.material.as_deref(), Some("球铁"));
        assert_eq!(config.pressure.as_deref(), Some("PN16"));
    }

    #[test]
    fn test_parse_empty() {
        let config = ConfigFields::parse("");
        assert!(config.is_empty());
    }

    #[test]
    fn test_parse_whitespace() {
        let config = ConfigFields::parse(" 执行标准 : 美标 , 压力 : 150LB ");
        assert_eq!(config.exec_std.as_deref(), Some("美标"));
        assert_eq!(config.pressure.as_deref(), Some("150LB"));
    }

    #[test]
    fn test_to_map() {
        let config = ConfigFields::parse("执行标准:美标,压力:150LB");
        let map = config.to_map();
        assert_eq!(map.get("执行标准").unwrap(), "美标");
        assert_eq!(map.get("压力").unwrap(), "150LB");
    }
}
