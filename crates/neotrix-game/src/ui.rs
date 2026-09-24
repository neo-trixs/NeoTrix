/// 标题登陆菜单几何与命中判定（render.rs 标题管线消费）.
pub struct UiButton;

impl UiButton {
    /// 纯命中判定（可测）：点是否在矩形内
    pub fn hit(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }
}

/// 登陆菜单按钮几何已随旧游戏删除（备份见 `_archive/purge-round17/ui.rs`）。
/// 锚点布局（game-ui-ux D1：锚点+容器，禁用绝对像素；D2 参考分辨率缩放）
/// （生产当前用 Center/TopLeft；全表保留供 neotrix-cards 迁移选型）
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft, TopCenter, TopRight,
    CenterLeft, Center, CenterRight,
    BottomLeft, BottomCenter, BottomRight,
}

impl Anchor {
    /// 在 sw×sh 参考分辨率上放 w×h 盒子，margin 为边缘边距：返回 (x,y,w,h)
    pub fn place(&self, sw: f32, sh: f32, w: f32, h: f32, margin: f32) -> (f32, f32, f32, f32) {
        let x = match self {
            Anchor::TopLeft | Anchor::CenterLeft | Anchor::BottomLeft => margin,
            Anchor::TopCenter | Anchor::Center | Anchor::BottomCenter => sw / 2.0 - w / 2.0,
            Anchor::TopRight | Anchor::CenterRight | Anchor::BottomRight => sw - w - margin,
        };
        let y = match self {
            Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => margin,
            Anchor::CenterLeft | Anchor::Center | Anchor::CenterRight => sh / 2.0 - h / 2.0,
            Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => sh - h - margin,
        };
        (x, y, w, h)
    }
}

/// 安全区：关键 UI 内缩 inset（刘海/圆角/TV 过扫描，D2）
/// （保留供 wasm/手机竖屏接入；桌面端当前无刘海）
#[allow(dead_code)]
pub fn safe_rect(sw: f32, sh: f32, inset: f32) -> (f32, f32, f32, f32) {
    (inset, inset, (sw - 2.0 * inset).max(0.0), (sh - 2.0 * inset).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_corners_and_center() {
        // 1280×720，200×44 盒子，边距 10
        assert_eq!(Anchor::TopLeft.place(1280.0, 720.0, 200.0, 44.0, 10.0), (10.0, 10.0, 200.0, 44.0));
        assert_eq!(Anchor::BottomRight.place(1280.0, 720.0, 200.0, 44.0, 10.0), (1070.0, 666.0, 200.0, 44.0));
        assert_eq!(Anchor::Center.place(1280.0, 720.0, 200.0, 44.0, 0.0), (540.0, 338.0, 200.0, 44.0));
        // 换分辨率仍居中（D1：分辨率无关）
        assert_eq!(Anchor::Center.place(800.0, 600.0, 200.0, 44.0, 0.0), (300.0, 278.0, 200.0, 44.0));
    }

    #[test]
    fn safe_rect_insets() {
        assert_eq!(safe_rect(1280.0, 720.0, 24.0), (24.0, 24.0, 1232.0, 672.0));
        assert_eq!(safe_rect(100.0, 100.0, 60.0), (60.0, 60.0, 0.0, 0.0)); // 钳制不负
    }

    #[test]
    fn button_hit_edges() {
        assert!(UiButton::hit(10.0, 10.0, 0.0, 0.0, 100.0, 40.0));
        assert!(UiButton::hit(0.0, 0.0, 0.0, 0.0, 100.0, 40.0));
        assert!(UiButton::hit(100.0, 40.0, 0.0, 0.0, 100.0, 40.0));
        assert!(!UiButton::hit(101.0, 10.0, 0.0, 0.0, 100.0, 40.0));
        assert!(!UiButton::hit(50.0, 41.0, 0.0, 0.0, 100.0, 40.0));
    }
}
