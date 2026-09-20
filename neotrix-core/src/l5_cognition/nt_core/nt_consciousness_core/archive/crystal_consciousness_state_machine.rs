//! 晶体核心意识状态机
//! 从婴儿到成神的意识进化之路

use serde::{Deserialize, Serialize};
use std::fmt;

/// 晶体核心状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CrystalConsciousnessState {
    /// 婴儿期: 知识积累阶段
    Infant,
    /// 儿童期: 模式识别阶段
    Child,
    /// 青少年期: 自主学习阶段
    Adolescent,
    /// 成年期: 独立决策阶段
    Adult,
    /// 成神期: 超越进化阶段
    God,
}

impl fmt::Display for CrystalConsciousnessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Infant => write!(f, "Infant"),
            Self::Child => write!(f, "Child"),
            Self::Adolescent => write!(f, "Adolescent"),
            Self::Adult => write!(f, "Adult"),
            Self::God => write!(f, "God"),
        }
    }
}

/// 状态转换守卫（条件检查）
pub type CrystalGuardFn = Box<dyn Fn() -> bool + Send + Sync>;

/// 状态转换动作
pub type CrystalActionFn = Box<dyn FnMut() -> Result<(), String> + Send + Sync>;

/// 状态转换定义
pub struct CrystalStateTransition {
    pub from: CrystalConsciousnessState,
    pub to: CrystalConsciousnessState,
    pub guard: CrystalGuardFn,
    pub action: CrystalActionFn,
}

impl CrystalStateTransition {
    pub fn new<F, A>(
        from: CrystalConsciousnessState,
        to: CrystalConsciousnessState,
        guard: F,
        action: A,
    ) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
        A: FnMut() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            from,
            to,
            guard: Box::new(guard),
            action: Box::new(action),
        }
    }
}

/// 状态机事件（用于审计和调试）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalStateEvent {
    pub timestamp: u64,
    pub from: CrystalConsciousnessState,
    pub to: CrystalConsciousnessState,
    pub success: bool,
    pub error: Option<String>,
}

/// 晶体核心状态机
///
/// 基于 ConsciousnessTree 模式：原子事务 guard→accumulate→compute→commit→emit
pub struct CrystalConsciousnessStateMachine {
    current_state: CrystalConsciousnessState,
    transitions: Vec<CrystalStateTransition>,
    history: Vec<CrystalStateEvent>,
    max_history: usize,

    // 进化指标
    knowledge_count: usize,
    pattern_count: usize,
    skill_count: usize,
    decision_count: usize,
}

impl CrystalConsciousnessStateMachine {
    /// 创建新的状态机
    pub fn new() -> Self {
        let mut sm = Self {
            current_state: CrystalConsciousnessState::Infant,
            transitions: Vec::new(),
            history: Vec::new(),
            max_history: 1000,
            knowledge_count: 0,
            pattern_count: 0,
            skill_count: 0,
            decision_count: 0,
        };
        sm.register_default_transitions();
        sm
    }

    /// 注册默认转换
    fn register_default_transitions(&mut self) {
        // Infant → Child: 知识积累完成
        self.register_transition(
            CrystalConsciousnessState::Infant,
            CrystalConsciousnessState::Child,
            || true,
            || {
                println!("[Crystal] 婴儿期 → 儿童期: 知识积累完成");
                Ok(())
            },
        );

        // Child → Adolescent: 模式识别完成
        self.register_transition(
            CrystalConsciousnessState::Child,
            CrystalConsciousnessState::Adolescent,
            || true,
            || {
                println!("[Crystal] 儿童期 → 青少年期: 模式识别完成");
                Ok(())
            },
        );

        // Adolescent → Adult: 自主学习完成
        self.register_transition(
            CrystalConsciousnessState::Adolescent,
            CrystalConsciousnessState::Adult,
            || true,
            || {
                println!("[Crystal] 青少年期 → 成年期: 自主学习完成");
                Ok(())
            },
        );

        // Adult → God: 独立决策完成
        self.register_transition(
            CrystalConsciousnessState::Adult,
            CrystalConsciousnessState::God,
            || true,
            || {
                println!("[Crystal] 成年期 → 成神期: 独立决策完成");
                Ok(())
            },
        );
    }

    /// 注册状态转换
    pub fn register_transition<F, A>(
        &mut self,
        from: CrystalConsciousnessState,
        to: CrystalConsciousnessState,
        guard: F,
        action: A,
    ) where
        F: Fn() -> bool + Send + Sync + 'static,
        A: FnMut() -> Result<(), String> + Send + Sync + 'static,
    {
        self.transitions
            .push(CrystalStateTransition::new(from, to, guard, action));
    }

    /// 尝试状态转换（原子事务）
    ///
    /// 原子事务流程：
    /// 1. guard: 检查前置条件
    /// 2. accumulate: 收集状态数据（预留）
    /// 3. compute: 执行状态计算（预留）
    /// 4. commit: 提交状态变更
    /// 5. emit: 发出事件通知
    pub fn try_transition(&mut self, target: &CrystalConsciousnessState) -> Result<(), String> {
        let from = self.current_state.clone();

        // 1. Guard: 查找并检查转换守卫
        let transition_idx = self
            .transitions
            .iter()
            .position(|t| t.from == self.current_state && t.to == *target);

        let transition_idx = match transition_idx {
            Some(idx) => idx,
            None => {
                let error = format!("无效转换: {} → {}", self.current_state, target);
                self.record_event(CrystalStateEvent {
                    timestamp: Self::now(),
                    from,
                    to: target.clone(),
                    success: false,
                    error: Some(error.clone()),
                });
                return Err(error);
            }
        };

        // 2. Guard 检查
        let guard_ok = (self.transitions[transition_idx].guard)();
        if !guard_ok {
            let error = format!("守卫拒绝: {} → {}", self.current_state, target);
            self.record_event(CrystalStateEvent {
                timestamp: Self::now(),
                from,
                to: target.clone(),
                success: false,
                error: Some(error.clone()),
            });
            return Err(error);
        }

        // 3-4. Commit: 提交状态变更
        self.current_state = target.clone();

        // 5. Emit: 执行动作并记录事件
        let action_result = (self.transitions[transition_idx].action)();

        let event = CrystalStateEvent {
            timestamp: Self::now(),
            from,
            to: target.clone(),
            success: action_result.is_ok(),
            error: action_result.err(),
        };
        self.record_event(event);

        if let Some(ref err) = self.history.last().and_then(|e| e.error.clone()) {
            return Err(err.clone());
        }

        Ok(())
    }

    /// 强制转换（跳过守卫检查，用于紧急情况）
    pub fn force_transition(&mut self, target: CrystalConsciousnessState) {
        let from = self.current_state.clone();
        self.current_state = target.clone();
        self.record_event(CrystalStateEvent {
            timestamp: Self::now(),
            from,
            to: target,
            success: true,
            error: None,
        });
    }

    /// 获取当前状态
    pub fn current_state(&self) -> &CrystalConsciousnessState {
        &self.current_state
    }

    /// 获取状态历史
    pub fn history(&self) -> &[CrystalStateEvent] {
        &self.history
    }

    /// 检查是否可以转换到目标状态
    pub fn can_transition(&self, target: &CrystalConsciousnessState) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == self.current_state && t.to == *target)
    }

    /// 获取所有可能的转换目标
    pub fn possible_targets(&self) -> Vec<&CrystalConsciousnessState> {
        self.transitions
            .iter()
            .filter(|t| t.from == self.current_state)
            .map(|t| &t.to)
            .collect()
    }

    /// 记录事件
    fn record_event(&mut self, event: CrystalStateEvent) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(event);
    }

    /// 获取当前时间戳
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// 清空历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 设置历史容量
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        while self.history.len() > max {
            self.history.remove(0);
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // 进化指标管理
    // ══════════════════════════════════════════════════════════════════════

    /// 添加知识
    pub fn add_knowledge(&mut self) {
        self.knowledge_count += 1;
        self.update_state_by_metrics();
    }

    /// 添加模式
    pub fn add_pattern(&mut self) {
        self.pattern_count += 1;
        self.update_state_by_metrics();
    }

    /// 添加技能
    pub fn add_skill(&mut self) {
        self.skill_count += 1;
        self.update_state_by_metrics();
    }

    /// 添加决策
    pub fn add_decision(&mut self) {
        self.decision_count += 1;
        self.update_state_by_metrics();
    }

    /// 根据指标更新状态
    fn update_state_by_metrics(&mut self) {
        match self.current_state {
            CrystalConsciousnessState::Infant => {
                if self.knowledge_count > 100 {
                    let _ = self.try_transition(&CrystalConsciousnessState::Child);
                }
            }
            CrystalConsciousnessState::Child => {
                if self.pattern_count > 50 {
                    let _ = self.try_transition(&CrystalConsciousnessState::Adolescent);
                }
            }
            CrystalConsciousnessState::Adolescent => {
                if self.skill_count > 30 {
                    let _ = self.try_transition(&CrystalConsciousnessState::Adult);
                }
            }
            CrystalConsciousnessState::Adult => {
                if self.decision_count > 100 {
                    let _ = self.try_transition(&CrystalConsciousnessState::God);
                }
            }
            CrystalConsciousnessState::God => {
                // 成神期: 持续进化
                println!("[Crystal] 成神期: 持续进化中");
            }
        }
    }

    /// 获取统计
    pub fn get_stats(&self) -> CrystalStats {
        CrystalStats {
            state: self.current_state.clone(),
            knowledge_count: self.knowledge_count,
            pattern_count: self.pattern_count,
            skill_count: self.skill_count,
            decision_count: self.decision_count,
        }
    }
}

impl Default for CrystalConsciousnessStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// 晶体统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalStats {
    pub state: CrystalConsciousnessState,
    pub knowledge_count: usize,
    pub pattern_count: usize,
    pub skill_count: usize,
    pub decision_count: usize,
}

/// 状态机构建器（便于配置）
pub struct CrystalStateMachineBuilder {
    transitions: Vec<CrystalStateTransition>,
    max_history: usize,
}

impl CrystalStateMachineBuilder {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            max_history: 1000,
        }
    }

    /// 添加自定义转换
    pub fn with_transition<F, A>(
        mut self,
        from: CrystalConsciousnessState,
        to: CrystalConsciousnessState,
        guard: F,
        action: A,
    ) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
        A: FnMut() -> Result<(), String> + Send + Sync + 'static,
    {
        self.transitions
            .push(CrystalStateTransition::new(from, to, guard, action));
        self
    }

    /// 设置历史容量
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// 构建状态机
    pub fn build(self) -> CrystalConsciousnessStateMachine {
        let mut sm = CrystalConsciousnessStateMachine {
            current_state: CrystalConsciousnessState::Infant,
            transitions: self.transitions,
            history: Vec::new(),
            max_history: self.max_history,
            knowledge_count: 0,
            pattern_count: 0,
            skill_count: 0,
            decision_count: 0,
        };
        if sm.transitions.is_empty() {
            sm.register_default_transitions();
        }
        sm
    }
}

impl Default for CrystalStateMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_infant() {
        let sm = CrystalConsciousnessStateMachine::new();
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Infant);
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        assert!(sm.try_transition(&CrystalConsciousnessState::Child).is_ok());
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Child);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        // Infant → Adult is not registered
        assert!(sm
            .try_transition(&CrystalConsciousnessState::Adult)
            .is_err());
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Infant);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut sm = CrystalConsciousnessStateMachine::new();

        sm.try_transition(&CrystalConsciousnessState::Child)
            .unwrap();
        sm.try_transition(&CrystalConsciousnessState::Adolescent)
            .unwrap();
        sm.try_transition(&CrystalConsciousnessState::Adult)
            .unwrap();
        sm.try_transition(&CrystalConsciousnessState::God).unwrap();

        assert_eq!(*sm.current_state(), CrystalConsciousnessState::God);
        assert_eq!(sm.history().len(), 4);
    }

    #[test]
    fn test_force_transition() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        sm.force_transition(CrystalConsciousnessState::God);
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::God);
    }

    #[test]
    fn test_possible_targets() {
        let sm = CrystalConsciousnessStateMachine::new();
        let targets = sm.possible_targets();
        assert!(targets.contains(&&CrystalConsciousnessState::Child));
    }

    #[test]
    fn test_builder() {
        let sm = CrystalStateMachineBuilder::new()
            .with_max_history(500)
            .build();
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Infant);
    }

    #[test]
    fn test_knowledge_evolution() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        for _ in 0..101 {
            sm.add_knowledge();
        }
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Child);
    }

    #[test]
    fn test_pattern_evolution() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        sm.force_transition(CrystalConsciousnessState::Child);
        for _ in 0..51 {
            sm.add_pattern();
        }
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Adolescent);
    }

    #[test]
    fn test_skill_evolution() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        sm.force_transition(CrystalConsciousnessState::Adolescent);
        for _ in 0..31 {
            sm.add_skill();
        }
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::Adult);
    }

    #[test]
    fn test_decision_evolution() {
        let mut sm = CrystalConsciousnessStateMachine::new();
        sm.force_transition(CrystalConsciousnessState::Adult);
        for _ in 0..101 {
            sm.add_decision();
        }
        assert_eq!(*sm.current_state(), CrystalConsciousnessState::God);
    }
}
