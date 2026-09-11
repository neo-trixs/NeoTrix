pub struct AnimationSystem {
    pub animations: Vec<Animation>,
}

pub struct Animation {
    pub id: String,
    pub frames: Vec<AnimationFrame>,
    pub current_frame: usize,
    pub elapsed: f32,
    pub looping: bool,
    pub playing: bool,
}

pub struct AnimationFrame {
    pub duration: f32,
    pub transform: Transform,
    pub color: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

impl AnimationSystem {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    pub fn play(&mut self, id: &str) {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.id == id) {
            anim.playing = true;
            anim.current_frame = 0;
            anim.elapsed = 0.0;
        }
    }

    pub fn stop(&mut self, id: &str) {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.id == id) {
            anim.playing = false;
        }
    }

    pub fn pause(&mut self, id: &str) {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.id == id) {
            anim.playing = false;
        }
    }

    pub fn resume(&mut self, id: &str) {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.id == id) {
            anim.playing = true;
        }
    }

    pub fn update(&mut self, dt: f32) {
        for anim in &mut self.animations {
            if !anim.playing || anim.frames.is_empty() {
                continue;
            }

            anim.elapsed += dt;
            let frame = &anim.frames[anim.current_frame];

            if anim.elapsed >= frame.duration {
                anim.elapsed -= frame.duration;
                anim.current_frame += 1;

                if anim.current_frame >= anim.frames.len() {
                    if anim.looping {
                        anim.current_frame = 0;
                    } else {
                        anim.current_frame = anim.frames.len() - 1;
                        anim.playing = false;
                    }
                }
            }
        }
    }

    pub fn get_current_frame(&self, id: &str) -> Option<&AnimationFrame> {
        self.animations
            .iter()
            .find(|a| a.id == id && a.playing)
            .and_then(|a| a.frames.get(a.current_frame))
    }

    pub fn is_playing(&self, id: &str) -> bool {
        self.animations
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.playing)
            .unwrap_or(false)
    }

    pub fn get_progress(&self, id: &str) -> f32 {
        if let Some(anim) = self.animations.iter().find(|a| a.id == id) {
            if anim.frames.is_empty() {
                return 0.0;
            }
            let total: f32 = anim.frames.iter().map(|f| f.duration).sum();
            let elapsed_in_frame = if anim.current_frame < anim.frames.len() {
                anim.elapsed
            } else {
                0.0
            };
            let before: f32 = anim.frames[..anim.current_frame].iter().map(|f| f.duration).sum();
            (before + elapsed_in_frame) / total
        } else {
            0.0
        }
    }
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_animation() -> Animation {
        Animation {
            id: "test".to_string(),
            frames: vec![
                AnimationFrame {
                    duration: 0.5,
                    transform: Transform {
                        x: 0.0,
                        y: 0.0,
                        ..Default::default()
                    },
                    color: Some(0xFF0000FF),
                },
                AnimationFrame {
                    duration: 0.5,
                    transform: Transform {
                        x: 10.0,
                        y: 10.0,
                        ..Default::default()
                    },
                    color: Some(0x00FF00FF),
                },
            ],
            current_frame: 0,
            elapsed: 0.0,
            looping: false,
            playing: false,
        }
    }

    #[test]
    fn test_new_animation_system() {
        let anim_sys = AnimationSystem::new();
        assert_eq!(anim_sys.animations.len(), 0);
    }

    #[test]
    fn test_add_and_play() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        assert!(!anim_sys.is_playing("test"));
        anim_sys.play("test");
        assert!(anim_sys.is_playing("test"));
    }

    #[test]
    fn test_stop() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        anim_sys.play("test");
        anim_sys.stop("test");
        assert!(!anim_sys.is_playing("test"));
    }

    #[test]
    fn test_update_advances_frame() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        anim_sys.play("test");
        anim_sys.update(0.3);
        assert_eq!(anim_sys.animations[0].current_frame, 0);
        anim_sys.update(0.3);
        assert_eq!(anim_sys.animations[0].current_frame, 1);
    }

    #[test]
    fn test_update_stops_at_end() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        anim_sys.play("test");
        anim_sys.update(2.0);
        assert!(!anim_sys.is_playing("test"));
        assert_eq!(anim_sys.animations[0].current_frame, 1);
    }

    #[test]
    fn test_looping() {
        let mut anim = make_test_animation();
        anim.looping = true;
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(anim);
        anim_sys.play("test");
        anim_sys.update(2.0);
        assert!(anim_sys.is_playing("test"));
        assert_eq!(anim_sys.animations[0].current_frame, 0);
    }

    #[test]
    fn test_get_current_frame() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        anim_sys.play("test");
        let frame = anim_sys.get_current_frame("test").unwrap();
        assert_eq!(frame.duration, 0.5);
    }

    #[test]
    fn test_get_current_frame_not_playing() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        assert!(anim_sys.get_current_frame("test").is_none());
    }

    #[test]
    fn test_progress() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.add(make_test_animation());
        anim_sys.play("test");
        let p = anim_sys.get_progress("test");
        assert!((p - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_nonexistent_animation() {
        let mut anim_sys = AnimationSystem::new();
        anim_sys.play("nope");
        assert!(!anim_sys.is_playing("nope"));
    }
}
