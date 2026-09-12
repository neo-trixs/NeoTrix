//! 浏览器 3D 预可视化 (NT-IO)
//!
//! 吸收源: github.com/NomaDamas/CozyClay
//! 成熟度: C1 (unit-tested stub, 无 Three.js 运行时集成)
//!
//! 核心能力: 把 3D 预可视化编排为场景摆位 + 镜头控制 + AI 导演接口。
//! 本 stub 负责场景摆位校验、镜头控制规格生成与导演指令校验。

use crate::core::nt_core_self_test::SelfTest;

/// 3D 摆位: 一个角色/物体在场景中的位置 (x,y,z) 与朝向。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StagePlacement {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 镜头: 摄像机位置 + 注视点。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CameraShot {
    pub cam_x: f64,
    pub cam_y: f64,
    pub cam_z: f64,
    pub look_x: f64,
    pub look_y: f64,
    pub look_z: f64,
}

/// 3D 预可视化 trait — 场景摆位 + 镜头控制 + AI 导演接口 stub。
pub(crate) trait CozyClay: Send + Sync {
    /// 校验摆位: 名称非空、坐标有限 (非 NaN/Inf)。重复名返回 None。
    fn validate_placement(&self, placements: &[StagePlacement]) -> Option<usize>;
    /// 生成镜头控制规格: 给定镜头返回 control 串, 坐标非有限返回 None。
    fn camera_control(&self, shot: &CameraShot) -> Option<String>;
}

/// 默认实现。
#[derive(Default)]
pub(crate) struct CozyClayPreviz;

impl CozyClay for CozyClayPreviz {
    fn validate_placement(&self, placements: &[StagePlacement]) -> Option<usize> {
        if placements.is_empty() {
            return None;
        }
        let mut seen = std::collections::HashSet::new();
        let all_finite = placements.iter().all(|p| {
            let finite =
                p.x.is_finite() && p.y.is_finite() && p.z.is_finite();
            let unique = !p.name.trim().is_empty() && seen.insert(p.name.as_str());
            finite && unique
        });
        if all_finite { Some(placements.len()) } else { None }
    }

    fn camera_control(&self, shot: &CameraShot) -> Option<String> {
        let all = [
            shot.cam_x, shot.cam_y, shot.cam_z,
            shot.look_x, shot.look_y, shot.look_z,
        ];
        if all.iter().all(|v| v.is_finite()) {
            Some(format!(
                "cozyclay://cam?pos={},{},{}&look={},{},{}",
                shot.cam_x, shot.cam_y, shot.cam_z,
                shot.look_x, shot.look_y, shot.look_z
            ))
        } else {
            None
        }
    }
}

/// T1 SelfTest: 验证摆位校验与镜头控制存在且生效。
#[derive(Default)]
pub(crate) struct CozyClaySelfTest;

impl SelfTest for CozyClaySelfTest {
    fn name(&self) -> &str {
        "nt_io_cozyclay"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let c = CozyClayPreviz;
        let placements = vec![
            StagePlacement { name: "hero".into(), x: 0.0, y: 0.0, z: 1.0 },
        ];
        match c.validate_placement(&placements) {
            Some(1) => {
                let shot = CameraShot {
                    cam_x: 2.0, cam_y: 1.0, cam_z: 3.0,
                    look_x: 0.0, look_y: 0.0, look_z: 1.0,
                };
                match c.camera_control(&shot) {
                    Some(s) if s.contains("cam") => Ok(()),
                    _ => Err(vec!["nt_io_cozyclay: camera control failed".into()]),
                }
            }
            _ => Err(vec!["nt_io_cozyclay: valid placement rejected".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_placement_ok() {
        let c = CozyClayPreviz;
        let ps = vec![
            StagePlacement { name: "a".into(), x: 1.0, y: 0.0, z: 0.0 },
            StagePlacement { name: "b".into(), x: -1.0, y: 0.0, z: 2.0 },
        ];
        assert_eq!(c.validate_placement(&ps), Some(2));
    }

    #[test]
    fn test_rejects_bad_placement() {
        let c = CozyClayPreviz;
        assert_eq!(c.validate_placement(&[]), None);
        let dup = vec![
            StagePlacement { name: "a".into(), x: 1.0, y: 0.0, z: 0.0 },
            StagePlacement { name: "a".into(), x: 2.0, y: 0.0, z: 0.0 },
        ];
        assert_eq!(c.validate_placement(&dup), None);
        let nan = vec![StagePlacement { name: "a".into(), x: f64::NAN, y: 0.0, z: 0.0 }];
        assert_eq!(c.validate_placement(&nan), None);
    }

    #[test]
    fn test_camera_control_spec() {
        let c = CozyClayPreviz;
        assert_eq!(c.camera_control(&CameraShot {
            cam_x: f64::INFINITY, cam_y: 0.0, cam_z: 0.0,
            look_x: 0.0, look_y: 0.0, look_z: 0.0,
        }), None);
        let s = c.camera_control(&CameraShot {
            cam_x: 1.0, cam_y: 2.0, cam_z: 3.0,
            look_x: 0.0, look_y: 0.0, look_z: 0.0,
        }).unwrap();
        assert!(s.starts_with("cozyclay://cam"));
    }
}
