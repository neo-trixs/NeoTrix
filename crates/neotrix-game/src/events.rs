// L1 事件系统 — TypeId 索引的类型安全事件总线
// 借鉴 Geese: 处理器按注册顺序执行，延迟队列 FIFO

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ── 类型擦除回调 ──

struct Handler {
    callback: Box<dyn Fn(&dyn Any)>,
}

/// 队列中的类型擦除事件
struct QueuedEvent {
    type_id: TypeId,
    data: Box<dyn Any>,
}

// ── 泛型 EventBus (TypeId 索引) ──

pub struct EventBus {
    /// 按事件类型 ID 索引的处理器列表
    handlers: HashMap<TypeId, Vec<Handler>>,
    /// 延迟事件队列 (类型擦除存储，附带 TypeId)
    queue: Vec<QueuedEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { handlers: HashMap::new(), queue: Vec::new() }
    }

    /// 注册事件处理器 — 编译期检查事件类型
    pub fn on<T: 'static>(&mut self, handler: impl Fn(&T) + 'static) {
        let type_id = TypeId::of::<T>();
        self.handlers.entry(type_id).or_default().push(Handler {
            callback: Box::new(move |any| {
                if let Some(event) = any.downcast_ref::<T>() {
                    handler(event);
                }
            }),
        });
    }

    /// 立即发射事件 — 编译期检查事件类型
    pub fn emit<T: 'static>(&self, event: T) {
        let type_id = TypeId::of::<T>();
        let any = &event as &dyn Any;
        if let Some(handlers) = self.handlers.get(&type_id) {
            for h in handlers { (h.callback)(any); }
        }
    }

    /// 延迟发射 — 入队，下帧 flush() 时处理
    pub fn enqueue<T: 'static>(&mut self, event: T) {
        self.queue.push(QueuedEvent {
            type_id: TypeId::of::<T>(),
            data: Box::new(event),
        });
    }

    /// 处理延迟队列 — FIFO 顺序，防风暴: 限制单次 flush 最大处理数
    pub fn flush(&mut self) {
        const MAX_FLUSH_PER_FRAME: usize = 64;
        let count = self.queue.len().min(MAX_FLUSH_PER_FRAME);
        let events: Vec<QueuedEvent> = self.queue.drain(..count).collect();
        for qe in events {
            if let Some(handlers) = self.handlers.get(&qe.type_id) {
                for h in handlers { (h.callback)(qe.data.as_ref()); }
            }
        }
    }

    pub fn queue_len(&self) -> usize { self.queue.len() }
    pub fn clear_queue(&mut self) { self.queue.clear(); }
    pub fn is_idle(&self) -> bool { self.queue.is_empty() }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}
