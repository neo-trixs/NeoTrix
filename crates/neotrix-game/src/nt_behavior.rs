//! Unity/Dreamlab 式 Behavior 脚本层（吸收 dreamlab-engine）.
//!
//! 动机：此前各游戏把逻辑手写进主循环（guixu `main.rs` 1263 行之其一根源），
//! Dreamlab 证明"实体＋可挂载 Behavior"是最小可用脚本抽象。本模块只做三件事：
//! 生命周期（attach/update/detach）＋注册表＋定序 tick；通信走组件（Godot
//! 信号思想的最小实现：共享组件即消息），网络复制走 M3 `Replicated<T>`（见
//! ROADMAP，由 Dreamlab networked-behaviors 启发，另起模块）。
//!
//! 约束：tick 内禁 attach/detach（快照语义，下一 tick 生效）；Behavior 禁 unsafe。

use std::collections::HashMap;

use crate::ecs::SimpleEcs;

/// 可挂载行为。`id` 为宿主实体；数据一律经 `SimpleEcs` 组件读写。
pub trait Behavior {
    /// 注册表内唯一名（同实体下按名开关/卸载）。
    fn name(&self) -> &'static str;
    fn on_attach(&mut self, _id: u64, _world: &mut SimpleEcs) {}
    fn on_update(&mut self, id: u64, world: &mut SimpleEcs, dt: f32);
    fn on_detach(&mut self, _id: u64, _world: &mut SimpleEcs) {}
}

struct Entry {
    behavior: Box<dyn Behavior>,
    enabled: bool,
}

/// 实体→行为表。`tick` 先快照 id（借用两阶段），再逐个驱动。
#[derive(Default)]
pub struct BehaviorRegistry {
    map: HashMap<u64, Vec<Entry>>,
}

impl BehaviorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 挂载并立即 `on_attach`（同名已存在则先替换旧者）。
    pub fn attach(&mut self, id: u64, world: &mut SimpleEcs, behavior: Box<dyn Behavior>) {
        let list = self.map.entry(id).or_default();
        if let Some(pos) = list.iter().position(|e| e.behavior.name() == behavior.name()) {
            let mut old = list.remove(pos);
            old.behavior.on_detach(id, world);
        }
        let mut entry = Entry { behavior, enabled: true };
        entry.behavior.on_attach(id, world);
        list.push(entry);
    }

    /// 按名卸载（不存在则无操作），触发 `on_detach`。
    pub fn detach(&mut self, id: u64, world: &mut SimpleEcs, name: &str) -> bool {
        let removed = match self.map.get_mut(&id) {
            Some(list) => match list.iter().position(|e| e.behavior.name() == name) {
                Some(pos) => Some(list.remove(pos)),
                None => None,
            },
            None => None,
        };
        match removed {
            Some(mut entry) => {
                entry.behavior.on_detach(id, world);
                if self.map.get(&id).map_or(false, Vec::is_empty) {
                    self.map.remove(&id);
                }
                true
            }
            None => false,
        }
    }

    /// 卸载实体全部行为（实体销毁时调用）。
    pub fn detach_all(&mut self, id: u64, world: &mut SimpleEcs) {
        if let Some(list) = self.map.remove(&id) {
            for mut entry in list {
                entry.behavior.on_detach(id, world);
            }
        }
    }

    pub fn set_enabled(&mut self, id: u64, name: &str, enabled: bool) -> bool {
        match self.map.get_mut(&id).and_then(|l| l.iter_mut().find(|e| e.behavior.name() == name))
        {
            Some(entry) => {
                entry.enabled = enabled;
                true
            }
            None => false,
        }
    }

    pub fn is_attached(&self, id: u64, name: &str) -> bool {
        self.map.get(&id).map_or(false, |l| l.iter().any(|e| e.behavior.name() == name))
    }

    pub fn count(&self, id: u64) -> usize {
        self.map.get(&id).map_or(0, Vec::len)
    }

    /// 定序驱动：实体 id 升序，同实体按挂载序；跳过死亡实体与禁用行为。
    pub fn tick(&mut self, world: &mut SimpleEcs, dt: f32) {
        let mut ids: Vec<u64> = self.map.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            if !world.is_alive(id) {
                continue;
            }
            if let Some(list) = self.map.get_mut(&id) {
                for entry in list.iter_mut().filter(|e| e.enabled) {
                    entry.behavior.on_update(id, world, dt);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Marker(&'static str);

    struct Rec {
        log: Rc<RefCell<Vec<String>>>,
        tag: &'static str,
    }

    impl Behavior for Rec {
        fn name(&self) -> &'static str {
            self.tag
        }
        fn on_attach(&mut self, id: u64, _w: &mut SimpleEcs) {
            self.log.borrow_mut().push(format!("attach:{}:{}", self.tag, id));
        }
        fn on_update(&mut self, id: u64, _w: &mut SimpleEcs, _dt: f32) {
            self.log.borrow_mut().push(format!("update:{}:{}", self.tag, id));
        }
        fn on_detach(&mut self, id: u64, _w: &mut SimpleEcs) {
            self.log.borrow_mut().push(format!("detach:{}:{}", self.tag, id));
        }
    }

    fn setup() -> (SimpleEcs, BehaviorRegistry, Rc<RefCell<Vec<String>>>) {
        (SimpleEcs::new(), BehaviorRegistry::new(), Rc::new(RefCell::new(Vec::new())))
    }

    #[test]
    fn lifecycle_order() {
        let (mut w, mut r, log) = setup();
        let id = w.spawn();
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        r.tick(&mut w, 0.016);
        r.tick(&mut w, 0.016);
        assert!(r.detach(id, &mut w, "a"));
        assert_eq!(
            *log.borrow(),
            vec!["attach:a:0", "update:a:0", "update:a:0", "detach:a:0"]
        );
    }

    #[test]
    fn disabled_skipped() {
        let (mut w, mut r, log) = setup();
        let id = w.spawn();
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        assert!(r.set_enabled(id, "a", false));
        r.tick(&mut w, 0.016);
        assert_eq!(*log.borrow(), vec!["attach:a:0"]);
    }

    #[test]
    fn multi_attach_order() {
        let (mut w, mut r, log) = setup();
        let id = w.spawn();
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "b" }));
        assert_eq!(r.count(id), 2);
        r.tick(&mut w, 0.016);
        assert_eq!(*log.borrow(), vec!["attach:a:0", "attach:b:0", "update:a:0", "update:b:0"]);
    }

    #[test]
    fn reattach_replaces() {
        let (mut w, mut r, log) = setup();
        let id = w.spawn();
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        assert_eq!(r.count(id), 1);
        assert_eq!(*log.borrow(), vec!["attach:a:0", "detach:a:0", "attach:a:0"]);
    }

    #[test]
    fn dead_entity_skipped() {
        let (mut w, mut r, log) = setup();
        let id = w.spawn();
        r.attach(id, &mut w, Box::new(Rec { log: log.clone(), tag: "a" }));
        w.despawn(id);
        r.tick(&mut w, 0.016);
        assert_eq!(*log.borrow(), vec!["attach:a:0"]);
    }

    #[test]
    fn reads_writes_components() {
        struct Grow;
        impl Behavior for Grow {
            fn name(&self) -> &'static str {
                "grow"
            }
            fn on_update(&mut self, id: u64, world: &mut SimpleEcs, dt: f32) {
                if let Some(m) = world.get_mut::<Marker>(id) {
                    m.0 = "grown";
                }
                let _ = dt;
            }
        }
        let (mut w, mut r, _) = setup();
        let id = w.spawn();
        w.insert(id, Marker("seed"));
        r.attach(id, &mut w, Box::new(Grow));
        r.tick(&mut w, 1.0);
        assert_eq!(w.get::<Marker>(id), Some(&Marker("grown")));
    }
}
